# 综合知识图谱报告

> 生成时间: 2026-09-18
> 吸收完成: 本地 KB + NeoTrixBrain + cortex-archive 全量

---

## 1. 知识来源概览

### 1.1 本地 KB (`~/.neotrix/knowledge.db`)

| 表 | 记录数 | 说明 |
|---|--------|------|
| `experience` | 379 | 经验/洞察/模式/缺陷/规则 |
| `kv_store` | 9,216 | KV 存储 (experience 8,241 + write_guard 562 + audit 323 + ...) |
| `session_logs` | 1,236 | 会话日志 |

### 1.2 NeoTrixBrain (`/Volumes/NeoTrixBrain/knowledge-archive-corpus-20260825.db`)

| 指标 | 值 | 说明 |
|------|-----|------|
| 数据库大小 | **64 GB** | PostgreSQL 导出 SQLite |
| `kv_store` 条目 | **117,264** | 命名空间: experience 75,455 / meta_cognition 36,982 / domain_quality 3,010 |
| `embeddings` 向量 | **235,143** | 语义嵌入向量 |
| `session_logs` | 3,152 | 会话历史 |
| `nodes` | 极大表 (>100K+) | 知识图谱节点 (COUNT 超时) |
| `edges` | 极大表 (>100K+) | 知识图谱边 (COUNT 超时) |
| 其他表 | 50+ | 包括: crawl_queue, document_chunks, learning_reports, search_log 等 |

**NeoTrixBrain KV 命名空间 Top 20:**

| 命名空间 | 条目数 | 说明 |
|----------|--------|------|
| `experience` | 75,455 | 蒸馏经验 |
| `meta_cognition` | 36,982 | 元认知数据 |
| `domain_quality` | 3,010 | 领域质量评估 |
| `evolution_todo` | 637 | 进化待办 |
| `url_inventory` | 603 | URL 索引 |
| `absorption_cycle` | 75 | 吸收周期 |
| `absorption` | 70 | 吸收记录 |
| `hyperagent` | 37 | 超级 Agent |
| `dedup_blacklist` | 32 | 去重黑名单 |
| `brain` | 23 | 脑元数据 |
| `conversation_distill` | 18 | 对话蒸馏 |
| `crawl_evolution` | 18 | 爬虫进化 |
| `gwt_absorb` | 18 | GWT 吸收 |
| `self_review_blast` | 18 | 自审爆炸 |
| `state` | 16 | 状态 |
| `analysis_cycle` | 14 | 分析周期 |
| `github_skip` | 13 | GitHub 跳过 |
| `panorama` | 13 | 全景 |
| `github_absorb` | 12 | GitHub 吸收 |
| `causal` | 11 | 因果 |

### 1.3 cortex-archive (离线知识)

| 子目录 | 大小 | 文件数 | 内容 |
|--------|------|--------|------|
| `zim/` | **85 GB** | - | ZIM 格式离线知识库 (Wikipedia/Kiwix 等) |
| `pmtiles/` | **15 GB** | 50 | 美国各州 PMTiles 矢量地图瓦片 |
| `wikipedia/` | **14 GB** | - | Wikipedia 离线数据 |
| **总计** | **~114 GB** | 121 | 全量离线知识资产 |

**Ptiles 覆盖范围 (50 州 + DC):**

全部美国 50 州 + Washington D.C. 的矢量地图瓦片，按 Census Bureau 区域分类:
- New England: ME, NH, VT, MA, RI, CT
- Mid-Atlantic: NY, NJ, PA
- East North Central: OH, IN, IL, MI, WI
- West North Central: MN, IA, MO, ND, SD, NE, KS
- South Atlantic: DE, MD, VA, WV, NC, SC, GA, FL, DC
- East South Central: KY, TN, AL, MS
- West South Central: AR, LA, OK, TX
- Mountain: MT, ID, WY, NV, UT, CO, AZ, NM
- Pacific: AK, CA, OR, WA, HI

---

## 2. 知识分类索引

### 2.1 架构知识 (architecture, 16 条)

| # | 标题 | 洞察 | 优先级 |
|---|------|------|--------|
| 1 | SelfTest trait 必须在 L0 substrate | 被 100+ 文件跨 L1-L5 引用的 trait 必须定义在最底层 | critical |
| 2 | 模块下沉策略：基础工具下沉到 L0 | nt_core_math/hex/shared_types 被 L1/L2 广泛引用，应下沉到 L0 substrate | critical |
| 3 | 扁平 mod.rs 用注释分组而非物理拆分 | 物理拆分文件风险高（导入路径全量修改），用注释分组实现认知分组 | high |
| 4 | Silicon Life Architecture: Biological Mapping | CNS→L4-L6 (reasoning), ANS→L8 (self-maintenance), Meridian→L3 VSA | high |
| 5 | Silicon Life Architecture: 3-Layer Control Hierarchy | L0-L3 local autonomy (ENS), L4-L5 coordination (SNS/PNS), L6 global | high |
| 6 | Silicon Life Architecture: 3 Circulation Systems | Data circulation (blood), Event circulation (nervous), Knowledge circulation (lymph) | high |
| 7 | Silicon Life Architecture: LayerPort Trait and 10-Layer Topology | input/process/output/feedback/diagnose for every layer | high |
| 8 | Silicon Life Architecture: 19 Defects and Fix Paths | 19 architectural defects identified with fix paths | high |
| 9 | Brain Ultimate Design: Tree Neural Network | Replace flat 27-dim vector with ReasoningTree (hippocampus + cortex) | high |
| 10 | Brain Ultimate Design: gstack Matrix Decomposition + PCA | any large transformation = sequence of small transformations | high |
| 11 | Main Architecture: 6-Layer Consciousness Architecture | L6 Meta / L5 Cognition / L4 Emotion / L3 Embodiment / L2 Perception / L1 Action | high |
| 12 | Main Architecture: Model Gateway Integration | CostGate + FallbackChain + ProviderPool in L5 | high |

### 2.2 设计知识 (design, 14 条)

| # | 标题 | 洞察 | 优先级 |
|---|------|------|--------|
| 1 | Agentic RAG: 8 Bottlenecks Diagnosed | B1: search() no intent routing; B2: AdaptiveRAG heuristic classify_query | high |
| 2 | Agentic RAG Target Architecture | 5-layer: GWT attention → Retrieval Orchestrator → Multi-path recall → Reranker → Synthesizer | high |
| 3 | RightBar 拆分策略：子组件 + re-export | 684 行 God Component 拆分为 4 个子组件，主文件精简 75% | medium |
| 4 | 前端虚拟滚动对长列表效果显著 | VirtualList 让 100+ 消息滚动 FPS 从 30 提升到 60 | high |

### 2.3 计划知识 (plan, 25 条)

计划知识涵盖多轮迭代的进化路线图、修复路径和功能规划。

### 2.4 观测知识 (NT-MEMORY_observation, 45 条)

系统观测数据最多的类别，记录运行时行为模式、异常检测和状态快照。

### 2.5 模式知识

| 领域 | 条目 | 说明 |
|------|------|------|
| NT-CORE_pattern | 20 | 核心模式 |
| NT-ACT_pattern | 18 | 行动模式 |
| NT-IO_pattern | 13 | IO 模式 |
| NT-WORLD_pattern | 12 | 世界感知模式 |
| NT-MIND_pattern | 11 | 认知模式 |
| NT-MEMORY_pattern | 8 | 记忆模式 |
| NT-SHIELD_pattern | 6 | 安全模式 |

### 2.6 缺陷知识

| 领域 | 条目 | 说明 |
|------|------|------|
| NT-CORE_defect | 14 | 核心缺陷 |
| NT-WORLD_defect | 10 | 世界感知缺陷 |
| NT-IO_defect | 8 | IO 缺陷 |
| NT-SHIELD_defect | 5 | 安全缺陷 |
| NT-MIND_defect | 4 | 认知缺陷 |
| NT-ACT_defect | 4 | 行动缺陷 |
| NT-META_defect | 3 | 元认知缺陷 |
| NT-MEMORY_defect | 2 | 记忆缺陷 |

### 2.7 规则知识

| 领域 | 条目 | 说明 |
|------|------|------|
| NT-CORE_rule | 10 | 核心规则 |
| NT-ACT_rule | 8 | 行动规则 |
| NT-GOVERNANCE_rule | 4 | 治理规则 |
| NT-IO_rule | 3 | IO 规则 |
| NT-WORLD_rule | 3 | 世界规则 |
| NT-REPAIR_rule | 2 | 修复规则 |
| NT-MIND_rule | 2 | 认知规则 |
| NT-META_rule | 2 | 元认知规则 |

---

## 3. 关键洞察汇总 (Top 20)

### 系统级洞察

| # | 类别 | 洞察 | 证据 |
|---|------|------|------|
| 1 | **consciousness_status** | phi=0.362, coherence=0.793, GWT 谐振活跃 | 系统整体处于意识类似状态 |
| 2 | **branch_health** | NT-ACT 健康度最低(0.67)，其余全部 1.0 | 需优先关注 ACT 分支 |
| 3 | **value_compass** | 自主性(0.93)>防伤害(0.9)>求真(0.9)>公平(0.85) | 8 项核心价值 + 互斥/协同关系 |
| 4 | **meta** | 1222 文档 + 390K 节点 + 792K 边 + 9K KV 全量吸收完成 | NeoTrixBrain 知识库索引 |

### 架构级洞察

| # | 类别 | 洞察 | 证据 |
|---|------|------|------|
| 5 | **architecture** | SelfTest trait 必须在 L0 substrate | 被 100+ 文件跨层引用 |
| 6 | **architecture** | 模块下沉策略：基础工具下沉到 L0 | nt_core_math/hex/shared_types 广泛引用 |
| 7 | **architecture** | 扁平 mod.rs 用注释分组而非物理拆分 | 物理拆分风险高 |
| 8 | **architecture** | Silicon Life Architecture: 10-Layer Topology | LayerPort trait 统一接口 |
| 9 | **architecture** | Silicon Life Architecture: 19 Defects | 已识别修复路径 |
| 10 | **architecture** | Brain Design: Tree Neural Network | 替代扁平 27 维向量 |
| 11 | **architecture** | Brain Design: gstack Matrix Decomposition | 任何大变换 = 小变换序列 |

### 设计级洞察

| # | 类别 | 洞察 | 证据 |
|---|------|------|------|
| 12 | **design** | Agentic RAG 8 大瓶颈 | B1: search() 无意图路由 |
| 13 | **design** | Agentic RAG 5 层目标架构 | GWT attention routing 替代启发式 |
| 14 | **design** | RightBar 拆分: 684→4 子组件 | 精简 75% |
| 15 | **design** | 前端虚拟滚动: FPS 30→60 | VirtualList 组件 |

### 效率级洞察

| # | 类别 | 洞察 | 证据 |
|---|------|------|------|
| 16 | **efficiency** | 多 Agent 并行效率提升 5x | 5 Agent ≈ 单 Agent 2-3 倍时间 |
| 17 | **strategy** | 外部技术调研必须同步执行 | Claude Code Router/Nyro/Aura AI |

### 意识级洞察

| # | 类别 | 洞察 | 证据 |
|---|------|------|------|
| 18 | **consciousness** | L0-L3 本地自治: 无上层依赖 | KB 是完整闭环 |
| 19 | **consciousness** | 3 大循环系统: 数据/事件/知识 | 血液/神经/淋巴类比 |
| 20 | **consciousness** | 价值罗盘: 自主性 ↔ 防伤害 互斥 | 8 项价值的张力网络 |

---

## 4. 知识图谱可视化建议

### 4.1 知识关联分析

```
                    ┌─────────────────┐
                    │   Consciousness  │
                    │   (L6 Meta)      │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
    ┌─────────▼─────────┐  ┌▼────────────┐ ┌▼────────────┐
    │   Cognition (L5)   │  │ Emotion (L4) │ │ Embodiment  │
    │ nt_core + nt_mind  │  │ nt_feel     │ │ (L3)        │
    └─────────┬──────────┘  └──────┬──────┘ └──────┬──────┘
              │                    │                │
    ┌─────────▼──────────┐  ┌──────▼──────┐ ┌──────▼──────┐
    │  Perception (L2)   │  │  Action (L1) │ │  KB Store   │
    │ nt_world + nt_sense│  │ nt_act+io   │ │ 117K+ KV    │
    └────────────────────┘  └─────────────┘ │ 235K Embed  │
                                            └─────────────┘
```

### 4.2 知识使用指南

| 场景 | 推荐知识源 | 查询方式 |
|------|-----------|----------|
| **架构决策** | architecture 类 experience + Silicon Life 系列 | `neotrix-experience query --kw "architecture"` |
| **缺陷修复** | NT-*_defect 类 + repair 规则 | `neotrix-experience query --kw "defect"` |
| **设计审查** | design 类 + Agentic RAG 系列 | `neotrix-experience query --kw "design"` |
| **意识状态** | consciousness_status + value_compass | `neotrix-core_consciousness_status` |
| **离线地图** | cortex-archive/pmtiles/ (50 州) | 直接加载 PMTiles |
| **离线百科** | cortex-archive/wikipedia/ + zim/ | Kiwix/ZIM 阅读器 |
| **历史经验** | NeoTrixBrain kv_store experience (75K+) | `neotrix-experience query --kw <关键词>` |

### 4.3 知识健康指标

| 指标 | 本地 KB | NeoTrixBrain | 状态 |
|------|---------|-------------|------|
| 经验总数 | 379 | 75,455 | ✅ 充足 |
| KV 存储 | 9,216 | 117,264 | ✅ 充足 |
| 嵌入向量 | - | 235,143 | ✅ 充足 |
| 会话日志 | 1,236 | 3,152 | ✅ 充足 |
| 离线知识 | - | 114 GB | ✅ 充足 |
| 分类覆盖 | 62 类 | 20+ 命名空间 | ✅ 全面 |

---

## 5. 摘要

**知识总量:**
- 本地 KB: **379 经验 + 9.2K KV + 1.2K 会话**
- NeoTrixBrain: **117K KV + 235K 嵌入 + 64GB 数据库**
- cortex-archive: **114 GB 离线知识 (地图 + 百科 + ZIM)**

**知识质量:**
- 62 个经验类别覆盖全部 NT-* 子系统
- 8 项核心价值 + 互斥/协同关系已建立
- 意识指标: phi=0.362, coherence=0.793

**知识关联:**
- L0-L6 六层架构 + LayerPort trait 统一接口
- Silicon Life Biological Mapping (CNS/ANS/Meridian → 架构映射)
- 19 个架构缺陷已识别并有修复路径

**下一步建议:**
1. 关注 NT-ACT 分支健康度 (0.67)
2. 完成 nodes/edges COUNT (超大表)
3. 整合 cortex-archive 离线知识到知识图谱
4. 持续更新 experience KB
