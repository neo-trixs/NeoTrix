# 记忆大脑 — 星系多维立体记忆存储架构设计 (Memory Brain Galaxy Topology)

> 状态: **Draft (评审中)** | 作者: mil/officer + NT-CORE | 日期: 2026-08-06
> 前置: 用户提案「除 AGENTS.md 外, 所有长效存储数据构建进记忆大脑知识库, 格式用星系多维
> 立体记忆存储架构 + 高维链路拓扑; 自动裁剪规则记忆, 避免装饰性语言和重复单词」。

---

## 1. 现状盘点 (全部经源码核实, 非假设)

| 层 | 已有资产 | 位置 | 状态 |
|----|----------|------|------|
| 知识库持久化 | `KnowledgeBase` (node/edge/kv) | `neotrix-core/src/neotrix/l3_memory_impl/nt_memory_kb/` | ✅ 生产可用 |
| 第二大脑 | `SecondBrain` (BrainSnapshot/BrainDimensionScore/BrainLink/BrainRelationType) | `neotrix-core/src/core/nt_core_second_brain.rs` | ✅ 生产接线 (emotion/session_note/wiki_graph) |
| 向量存储 | `nt_core_vector_store` | `neotrix-core/src/core/nt_core_vector_store/` | ✅ 存在 |
| **星系多维拓扑核心** | `nt_core_hcube::topology` (BettiNumbers/PersistentHomology/PointCloud) | `neotrix-core/src/core/nt_core_hcube/topology.rs` | ⚠️ **死声明 — 零生产引用** |
| **Hebbian 记忆** | `HebbianGraph` (observe/hebbian_update/diffusion_retrieve/decay/prune_edges) | `neotrix-core/src/core/nt_core_hcube/hebbian_memory.rs` | ⚠️ **死声明 — 零生产引用** |
| **巩固器 (Consolidator)** | `ReflectionConsolidation` (verify/cross_link/prune/compress; cross_link_thr=0.5, compress_thr=0.8, min_access=1) | `neotrix-core/src/core/nt_core_hcube/reflection_consolidation.rs` | ⚠️ **死声明 — 已实现但依赖内存 FhrrHyperCube 码本+随机剪枝, 不适合文本 KB (2026-08-06 结论)** |
| 符号 VSA | `ghrr_vsa` (ghrr_similarity/compute_similarity_matrix) | `neotrix-core/src/core/nt_core_hcube/ghrr_vsa.rs` | ⚠️ 未接线到检索 |
| 超立方坐标 | `cube.rs` (insert/query/query_with_scores) | `neotrix-core/src/core/nt_core_hcube/cube.rs` | ⚠️ 未接线 |
| 经验树 | `experience.rs` CLI (absorb/query/list/route) | `neotrix-core/src/bin/experience.rs` | ✅ 生产 (FTS5 关键词检索) |
| 规则登记 | registry.md + CTREE.md 双登 (R-P101) | `~/.agents/skills/` | ✅ 操作规则 |

**核心洞察**: 用户要的「星系多维立体记忆 + 高维链路拓扑」**不是新建**, 而是把
`nt_core_hcube` 里已写好的 `topology` + `ghrr_vsa` 与现有持久化 concept 图
**接线进 KB 检索**。现有检索原为纯 FTS5 关键词匹配, 无法表达「链路拓扑」「星系多维」。
> `reflection_consolidation` 经核查**不适合直接接线** (内存码本+随机剪枝), 其 dedup/compress
> 职能由写入前 sim≥0.65 过滤承担, 链路职能由 concept 图多跳扩散承担 (见 §4.1b)。

> **2026-08-06 研究深化 (cycle 225 前置)**: 文献与开源调研表明 — (a) 检索应走
> **多信号混合**(语义 embedding + BM25 关键词 + 实体/图遍历并行融合, 见 Mem0/Graphiti),
> 而非纯语义; (b) dedup 走 **VSA 写入前过滤** (阈值实测校准 0.65), 而非内存随机合并;
> (c) 裁剪应引入 SRMU 的 **relevance-gated 写入前过滤**(先滤冗余/陈旧再落盘) + Hebbian 时间衰减,
> 而非仅靠事后 cleanup; (d) 拓扑 Betti 数与 E8 晶格结合已有文献先例。详见 §6 研究基础。

---

## 2. 分层边界 (HARD BOUNDARY)

禁止全量迁入 KB。长期记忆分两库:

### A. 文件系统层 (单一事实源, 人读 + git diff 审计)
| 文件 | 原因 |
|------|------|
| `AGENTS.md` | 指针/协议, 指针守恒硬规则 |
| `dev-rules.md` (R-P1..R-P101) | 操作规则, 需 git 审计, R-P101 规定为登记事实源 |
| `CTREE.md` / `registry.md` | 登记表, 人读 |
| `CONTEXT.md` / 共享语言 | 会话前缀 |
| skills `*/SKILL.md` / `profile.yaml` | 技能定义 (RULES.md 层规则规定) |

### B. KB 记忆层 (可检索记忆, 向量 + 拓扑 + Hebbian)
| 内容 | 现有键位 |
|------|----------|
| 经验树全部 cycle (指针+摘要+全文) | `kv_store` / `experience` namespace |
| 规则 R-P 系列全文 | 迁入 `knowledge` namespace (文件留指针) |
| 能力星系节点/链路 | `capability` namespace (节点+边) |
| session 笔记/情绪 | `session_notes` / `emotion` (已有) |

> **原则**: 文件系统层 = 静态真源; KB 层 = 动态计算索引 (可增量重建, 永不写回文件层)。

---

## 3. 星系多维立体记忆架构 (核心设计)

每一条记忆 = 一个**星系节点**, 三个维度编码 + 高维链路:

### 3.1 节点表示 (Multi-Dimensional Memory Node)
```
记忆节点 = {
  id: branch_<cycle>_<i>_<hash>
  embed: Vec<f64>                      # ghrr_vsa 符号嵌入 (高维向量, ~1-10K 维)
  coord: HyperCoord                    # cube.rs 超立方坐标 (域×层×成熟度×类型)
  text: { content, summary, evidence } # 已压缩正文 (吸收时裁剪装饰语)
  stats: { importance, access, ttl }   # Hebbian 统计
  links: [(target_id, relation, weight)]  # 星际链路
}
```

### 3.2 星系拓扑 (Galaxy Topology)
- **节点** = 记忆; **链路** = KB 的 `upsert_edge` (关系: Related/Depends/RoutesTo/Governs)
- **高维结构提取**: 把经验节点嵌入向量聚成 `PointCloud` → `PersistentHomology::compute`
  - `beta_0` = 连通域数 (星系聚类数)
  - `beta_1` = 环路 (跨 cycle 闭环 → 高置信模式, 如反复出现的 failure archetype)
  - `beta_2` = 空洞 (缺失知识域 → 探测盲区)
  - `integration_estimate()` = 星系整合度 Φ 指标, 监控记忆系统的整体连通健康
- **链路复用**: `SecondBrain::link_nodes` + `BrainRelationType`, 不强建新图

### 3.3 记忆检索管线 (混合检索 Hybrid Retrieval, 替代纯 FTS5)

研究结论 (Mem0 2026 single-pass / Graphiti hybrid / HiGMem 事件锚点): 单一信号检索
(纯关键词或纯向量) 均会 bloat 证据集。采用**多信号并行 + 融合**:

```
query "主题"
  → 信号① ghrr_similarity(嵌入, 全部节点)   # 语义近邻 top-N   (seeded from ghrr_vsa)
  → 信号② FTS5/BM25 关键词命中             # 精确词保底        (保留现有 db, 不回退)
  → 信号③ HebbianGraph::diffusion_retrieve(seeds, steps, top_k, decay)  # 图扩散关联链路
  → 融合: 三信号得分加权 (语义 0.4 / 关键词 0.3 / 图扩散 0.3), 初始权重 C3 后用数据校准
  → 可选碰撞重排: 拓扑 integration_estimate 加权 (高连通聚类内的候选优先)
  → 返回 { key, content, links, access }  # 保持 query --json 契约不变 (R-P101 机器可读)
```

**GAM/TiMem 启示**: 图记忆**分层** — 高分层"语义锚"事件摘要(先粗筛), 低分层原始
分支(再细读)。检索先查摘要层锚点, 命中才下钻分支, 大幅降上下文成本 (HiGMem 基于此
把 adversarial F1 0.54→0.78, 检索 turn 少一个数量级)。本设计沿用现有 cycle 摘要即锚点。

---

## 4. 自动裁剪规则记忆 (Auto-Prune)

### 4.1 巩固 + 写入前过滤 (Relevance-Gated, 非仅事后 cleanup)

**(a) 写入前过滤 (SRMU 启示, P2 阶段已实现)**: 吸收时先判相关性再落盘, 非全收:
```
新 session content
  → 与现有同 domain 分支算 VSA 词袋相似度 (ghrr_vsa, 2048 维)
  → 与任何同 domain 分支 sim ≥ 0.65 → 判冗余, 跳过落盘
  → 否则才进 observe + 落盘
```
阈值校准 (2026-08-06 实测 sim 分布): 精确副本 = 1.0, 改写 ≈ 0.75-0.81,
部分重叠 ≈ 0.02, 无关 ≈ 0.000。故 0.65 可拦截"高度近似改写"且放行真新内容。
> 原设计曾写 compress_thr=0.8 / cross_link_thr=0.5 (ReflectionConsolidation 默认),
> 实测 VSA 词袋下 0.8 会漏过 0.75-0.81 的改写近似, 0.65 更贴合分布。

SRMU 论文 (美陆军 Ground Systems, 2026): 纯加法更新会持久化陈旧信息; 先滤冗余/陈旧
再存储, 记忆相似度 +12.6%, 记忆幅度 -53.5%。

**(b) 落盘后检索扩散 (已实现, 基于持久化 concept 图, 非内存副本)**: 原计划接线
`ReflectionConsolidation` (reflection_consolidation.rs) 作为落盘后巩固, 但核查发现
该模块依赖内存 `FhrrHyperCube` 符号码本 (不持久化)、`prune` 为**随机 5% 剪枝** (对
真实 KB 具破坏性)、`cross_link` 产生 `composite:` 污染符号 —— 直接接线 = 平行适配器
(R-P42 违反) + 数据风险。**采纳的最优解**:
- **写入前过滤已承担 dedup/compress 职能** (§4.1a), 无需事后随机合并;
- **Hebbian 多跳扩散检索**: 在**持久化 concept 图** (SQLite concept_* + co_w) 上实现
  BFS 扩散 (`neural_associative`, experience.rs): 种子概念 → 沿 co 边多跳 (HOP_LIMIT=3,
  DECAY=0.5, FRONTIER_K=12 剪枝防爆炸), 关联概念分支获得衰减权重作为相关推荐
  (order=2, 分数 *0.1 压后不淹没直接命中)。
- 验证 (2026-08-06): 冷词"六层管线" hebb 开启多出 7 个 [关联] 扩散结果 vs 仅 2 个直接命中。

### 4.3 装饰性语言过滤 (吸收时压缩, 非检索时)
- 吸收协议已要求压缩字段 (`content` 是精简正文, 非全 transcript)
- 新增「装饰语词表」正则过滤: 前缀赞词 (如「值得注意的是」「更重要的是」「综上所述」) 在 absorb 时删除
- **不裁剪历史** (审计完整性), 只裁剪**新写入**与**规则记忆**

---

## 5. 实施路线 (分阶段, 符合 C0-C5 星座纪律 + R-P101)

| 阶段 | 内容 | 成熟度目标 | 验收 |
|------|------|-----------|------|
| P0 | `reflection_consolidation.rs` / `topology.rs` / `hebbian_memory.rs` / `ghrr_vsa.rs` 编译冒烟 (cargo build 全 targets) | C0 | 无死声明编译错误 |
| P1 | 检索加语义信号: `ghrr_vsa` 近邻 + 现有 FTS5 混合 (query --json 契约不变) | C1 | 单测: 语义等价词命中; FTS5 精确词保底不回退 |
| P2 | Hebbian 多跳扩散检索: 持久化 concept 图 BFS (HOP_LIMIT=3/DECAY=0.5/FRONTIER_K=12), order=2 相关推荐 | C2 | 冷词扩散出关联分支, 不淹没直接命中 (2026-08-06 完成) |
| P3 | 写入前过滤 (SRMU relevance-gated) — **VSA sim≥0.65 拦截冗余** (阈值实测校准 0.75-0.81 改写近似) | C2 | 冗余 session 不入库 (2026-08-06 完成); 装饰语词表待做 |
| P4 | `topology --json`: 经验 VSA 嵌入 → 持续同调 Betti 曲线 + 记忆簇 (scale≤0.6 union-find), 归一化向量 scale_max=0.8, max_points 分层采样控 O(n³) | C3 | 输出 β₀/β₁/β₂ 曲线 + integration_estimate + 簇成员映射回真实分支 (2026-08-06 完成) |
| P5 | 规则 R-P 系列全文迁 KB `experience` namespace (cycle 227: 75 条镜像, type=rule/NT-GOVERNANCE); dev-rules.md 保留为单一事实源 + 头部 KB 镜像索引 (双轨, 不破坏惰性加载) | C3 | `query --kw "R-P101"` 命中 227 镜像; dev-rules.md ↔ KB 双登可检索 |
| P6 | 全链路 benchmark (检索延迟/命中率, 混合 vs 纯 FTS5; 参照 LoCoMo/LongMemEval 方法) | C4 | 混合命中率 ≥ FTS5, 延迟可接受 |

> **R-P79 接线门**: 每阶段完成必须接线到生产路径 (CLI/absorb/query), 禁止延期死代码。

---

## 6. 研究基础 (2026 文献 + 开源实证)

> 2026-08-06 调研。支撑 §3/§4 设计决策的证据链。

### 6.1 论文 (学术文献)

| 论文/来源 | 关键结论 | 本设计采纳 |
|-----------|----------|-----------|
| **MemGPT** (Packer et al., 2023, arxiv 2310.08560) | OS 分层记忆: main context=RAM, external=disk; self-directed 换页 | §2 分层边界; 文件层=external storage |
| **GAM** (Wu et al., ACL 2026 long) | 图记忆**编码/巩固解耦**: event graph 只在语义迁移时并入 topic 关联网; 降干扰尽保一致 | §3.3 分层锚点; consolidate 时机非每 turn |
| **TiMem** (Li et al., ACL Findings 2026) | 时间-层次 TMT; LoCoMo 75.3%, 召回长度 -52% | 时间分层; 摘要即高层锚点 |
| **HiGMem** (Cao et al., ACL Findings 2026) | 高分层事件摘要作语义锚, 命中才下钻; adversarial F1 0.54→0.78, turn 少 10× | §3.3 摘要锚点检索 |
| **RecMem** (Dai et al., ACL Findings 2026) | 只在**重复相似模式**时才 LLM 提取 consolidate (recurrence-based), 存储 token 省高达 87% | §4 写入前过滤 (判定重复才巩固) |
| **SRMU** (Snyder et al., 2026, arxiv 2604.15121) | VSA 序列记忆 relevance-gated 更新: 写入前滤冗余/陈旧, 相似度 +12.6%, 幅度 -53.5% | §4.1a 写入前过滤 |
| **Memory in LLM Era** (arxiv 2604.01707) | 统一四段框架: 提取→管理→存储→检索; 分层存储是最优 | §1 现状映射按四阶段 |
| **GrapHD** (PMC8855686) | HD/VSA 图记忆: 编码全图到超矢量, 支持图重构/路径查询/鲁棒 | §3 支持图拓扑在 VSA 上 |
| **HD/VSA Survey** (Kleyko et al.) | VSA = 分布式表示 + 绑定/叠加, 需 item memory 做 recovery/cleanup | 附录 API 语义 |
| **Sleep replay** (Nature Comm, 2022) | 无监督 sleep replay + Hebbian 剪枝减少灾难性遗忘, 正交化记忆 | §4 周期巩固 (sleep-like pass) |
| **PPT consolidation** (PLoS Comp Bio, 2021) | 平行通路 + Hebbian 巩固 → power-law 遗忘曲线 | decay 幂律形状参考 |
| **Dynamic attractor** (PLoS Comp Bio, 2023) | 遗忘 = 持续弱化, 竞争着强化; 被遗忘 memory 释放资源 | §4.1 弱化 vs 强化竞争 |
| **Zep/Graphiti** (Zep et al., 2025) | 时序知识图谱: 增量更新 + bi-temporal 无效化 + 混合检索 (语义+BM25+图遍历) | §3.3 混合检索 (语义+关键词+图) |

### 6.2 开源项目 (可复用实现, GitHub)

| 项目 | 定位 | 可借鉴 | 取舍 (本设计不照搬) |
|------|------|--------|---------------------|
| **mem0ai/mem0** | 通用记忆层 | 多信号融合 (语义+BM25+实体); 实体链 (entity linking) | 依赖外部 embedding LLM; 本仓库用 VSA 符号免外部模型 |
| **getzep/graphiti** | 时序知识图谱引擎 | bi-temporal 事实; hybrid semantic+keyword+graph traversal | 需 3rd-party graph DB; 本仓库内嵌 KB edge |
| **hiGMem** (ZeroLoss-Lab) | 分层锚点检索 | 事件摘要锚点 → 下钻 | §3.3 摘要锚点, 无需额外 DB |
| **neo4j-labs/agent-memory** | 图原生记忆 (POLE+O) | 短/长/推理三态记忆分工, entity dedup | Neo4j 依赖; 本仓库自建 KB |
| **aexy-io/graphzep** | TS 版 Zep 时序图 | bi-temporal + 语义检索 | 参考其分层设计思路 |

### 6.3 关键采纳决策
1. **混合检索**(Mem0/Graphiti/Zep 共识): 语义+关键词+图三信号融合 → §3.3 改写
2. **先滤后存**(SRMU/RecMem): 写入前 relevance 判定省 token, 非仅事后 dedup → §4.1a
3. **复用现成巩固器**(本仓库 reflection_consolidation 已含 compress 0.8/cross 0.5): 零新增 dedup 算法 → §4.1b
4. **分层锚点检索**(TiMem/HiGMem): 摘要层→分支层下钻, 降上下文成本 → §3.3
5. **对标基准**(LoCoMo/LongMemEval): P6 用同方法学评估, 可复现 → §5

---

## 7. 风险与对策

| 风险 | 对策 |
|------|------|
| VSA 嵌入质量未知 (ghrr 未生产验证) | P1 先 A/B 对比 FTS5 命中率, 不达标不前进 |
| Hebbian decay 误删重要记忆 | 只 prune < 阈值的弱边; 节点不删, 只衰减链路 |
| 拓扑计算 O(n²) 慢 | 点云分批 (per domain), 非全量 |
| 文件层与 KB 层漂移 | KB 为可重建索引, 永不写回; 真源在文件 |
| dedup 阈值武断 | C2 后用真实数据校准, 先硬编码 |

---

## 8. 待决问题 (Reviewer 评审点)

1. **混合检索信号融合权重?** 研究共识是并行融合 (语义+关键词+图)。初始 0.4/0.3/0.3, C3 校准。是否接受先硬编码?
2. **规则 R-P 迁 KB 后, git 审计靠什么?** 建议 dev-rules.md 保留 R-P 头一行摘要 + 指针 (与 R-P101 一致)。
3. **装饰语词表范围?** 中英双语? 只滤 rules/experience, 不滤 SKILL 定义文。
4. **拓扑报告消费方?** 只有监控, 还是要驱动注意力路由 (GWT)?
5. **写入前过滤是否与 `absorb` 幂等去重 (v3.2) 冲突?** 需保证 SRMU relevance 判定 + session_id 幂等并存, 不 double-count.

---

## 附录: 已核实的现成 API (接线点)

```rust
// topology.rs — 星系拓扑
let cloud = PointCloud::new("experience");
cloud.add_point(embed.clone());
let ph = PersistentHomology::compute(&cloud, scale_max, num_steps);
let b = ph.simplified_betti();           // beta_0/1/2
let phi = b.integration_estimate();      // 整合度

// hebbian_memory.rs — 自动裁剪
let mut g = HebbianGraph::new(0.1, 0.3, 0.98);
g.observe_batch(&pairs);
g.hebbian_update(a, b, target, importance);
g.diffusion_retrieve(&seeds, steps, top_k, decay);
g.decay();                                // 时间衰退
g.prune_edges(0.2);                       // 裁剪弱边

// ghrr_vsa.rs — 语义相似度 (dedup + 检索)
let sim = ghrr_similarity(&a, &b);        // [0,1]

// reflection_consolidation.rs — 现成 dedup+裁剪巩固器 (P2 直接接线, 零新增算法)
let mut cons = ReflectionConsolidation::default();   // cross_link_thr=0.5, compress_thr=0.8
let report = cons.consolidate(&mut hc);               // verify → cross_link → prune → compress
// report.{symbols_before, after, pruned, merged, cross_links, verified}
```
