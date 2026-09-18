# NeoTrix 架构迭代路线图 — 2026-09-10

## 一、外部技术吸收摘要

### 1.1 多模型路由 (Cost-Aware)

| 来源 | 项目 | 关键洞察 | NeoTrix 映射 |
|------|------|----------|-------------|
| arXiv 2608.25992 | **ProgRouter** (EMNLP 2026) | 在线进度引导路由：多视图任务进度评分 + 双路径预测器 + 自适应元门控。按工作流步骤路由（非按查询）。ScienceWorld 上降低 58.7% 成本。 | **GWT salience + cost weight** — salience 评分加入 `task_progress` 信号。步骤级路由，非会话级。 |
| arXiv 2604.23530 | **MTRouter** | 历史-模型联合嵌入：编码交互历史 + 候选模型到共享空间，从日志轨迹学习结果估计器。HLE 上降低 43.4% 成本。 | **SelfModel 扩展** — 添加 `history_embedding` 字段。轻量 MLP (64-d) 映射 (history, model_id) → 预期结果。 |
| arXiv 2602.21227 | **BAAR** | 预算感知路由：Easy/Hard/Intractable 任务分类 + 预算约束解码。推理时剪枝违反预算的动作。 | **GWT 预算执行** — salience 上下文加 `budget_remaining`。预算 < 20% 强制路由到廉价模型。 |
| arXiv 2609.00662 | **Drift-Aware** | 非平稳工作负载处理：滚动审计窗口 + 悲观奖励 + 乐观成本 + 影子价格 + 硬承诺前计量。 | **NT-IO provider drift 检测** — 监控每 provider 延迟/成本/质量漂移，自动切换。 |
| Trendshift | **OmniRoute** (17.9K★) | 单端点路由 231+ providers, 50+ free。Token 压缩 + 智能 fallback + 多模态 API。 | 扩展 Ordered Backend Router (R-P82)。 |

### 1.2 上下文窗口管理

| 来源 | 项目 | 关键洞察 | NeoTrix 映射 |
|------|------|----------|-------------|
| arXiv 2609.04852 | **KVMem** | Paged KV 虚拟化：GPU→Host→NVMe 三级 KV 存储。模型原生 attention-space 索引。24GB GPU 上 1M token。 | 扩展 `kv_cache_optimizer.rs` — 实现 3 级 KV 放置 + Mean-K 块向量相关性评分。 |
| arXiv 2609.00749 | **ContextPipe** | 数据库启发的上下文组装：Plan→Bind→Optimize→Execute→Feedback 五阶段。31% token 减少。 | **GWT 上下文组装** — 将上下文窗口视为"查询预算"，每次组装 = 带 EXPLAIN 的查询执行。 |
| arXiv 2603.09023 | **Pichay** | L1-L4 需求分页：故障驱动钉住（内容被重请求 → 提升到 L2）。协作清理标签。 | **NT-MEMORY L1-L4 层级** — L1=活跃上下文, L2=工作集(故障钉住), L3=会话历史(有损压缩), L4=跨会话 KB。 |
| arXiv 2602.22402 | **CMV** | DAG 会话状态模型。三遍结构无损剪枝：剥离原始工具输出/元数据，保留所有用户+助手消息。20-86% 减少。 | **nt_nexus 会话 DAG** — 会话历史建模为 DAG，快照/分支/剪枝原语。 |
| arXiv 2607.25066 | **ARC** | Addressable Recall Compaction：追加只存 + 有界活跃视图 + 引用指针。观察无损：99.40% 准确率。 | **nt_memory::addressable_store** — 追加只存工具观察，活跃视图用引用替换旧观察。 |
| **context-mode** (17K★) | **Context Mode MCP** | 沙箱工具输出：315KB→5.4KB (98% 减少)。"Think in Code"范式。FTS5+BM25 会话连续性。 | **nt_io::context_sandbox** — 所有工具输出沙箱执行，仅摘要进入上下文。 |

### 1.3 技能/插件架构

| 来源 | 项目 | 关键洞察 | NeoTrix 映射 |
|------|------|----------|-------------|
| arXiv 2608.29596 | **Agentic Skills** | 9 阶段技能生命周期：发现→创作→存储→检索→路由→组合→执行→适应→评估→安全。 | **形式化 NT 技能生命周期** — 映射到 SEAL pipeline。 |
| arXiv 2609.09233 | **Subagents vs Skills** | 子代理执行（新鲜上下文）> 技能加载（共享上下文），当技能有清晰 I/O 契约时。 | **GWT 技能路由** — 复杂技能(Keystone 级) → spawn 子代理；简单技能(Small Passive 级) → 加载到当前上下文。 |
| arXiv 2609.09153 | **Procedural Graphs** | 自进化执行结构：(procedure, relation, procedure) 三元组。LLM 精炼器从失败/成功轨迹编辑图拓扑。 | **NT-MIND SEAL 进化** — 技能依赖图拓扑编辑。失败轨迹 → 编辑图 → 验证 → 提交/拒绝。 |
| arXiv 2609.09395 | **State-Path Tool Menus** | 菜单作为执行先验：检索器覆盖可执行入口 + 缺失输入生产者 + 最终动作。32 工具+状态路径 > 128 工具无状态路径。 | **GWT 工具菜单** — 按状态路径排序：入口→前置条件→最终动作。减少 75% 工具表面。 |
| Trendshift | **Eve** (Vercel, 4.9K★) | 文件系统优先 agent 框架：`agent/tools/` (类型化函数), `agent/skills/` (按需 .md)。 | **NT 技能目录约定** — 对齐 Eve 模式。 |

### 1.4 自进化/元认知

| 来源 | 项目 | 关键洞察 | NeoTrix 映射 |
|------|------|----------|-------------|
| Trendshift | **Prime Agent** (1.4K★) | "Continual Harness"：`/refine` 审查轨迹，应用小的、有证据支持的状态更新。从不重写基础系统提示。基于快照回滚。 | **nt_mind::continual_harness** — 添加 `/refine` 能力。小更新，快照回滚。 |
| arXiv 2609.09153 | **Procedural Graphs** | 自进化：LLM 精炼器对比失败/成功轨迹，编辑图拓扑。保留被拒绝编辑以防止重复。 | **NT-MIND SEAL** — 添加图拓扑编辑到 distillation 阶段。 |

### 1.5 桌面端架构

| 来源 | 项目 | 关键洞察 | NeoTrix 映射 |
|------|------|----------|-------------|
| Trendshift | **Cherry Studio** (42K★) | AI 生产力工作室：智能聊天、自主代理、300+ 助手。 | **NT-IO 桌面 UX** — 研究助手管理、会话持久化、多模型切换 UX。 |
| Trendshift | **agent-manager** | 终端 UI 管理 tmux 中的 AI 编码会话：实时状态、分组树、资源仪表。 | **NT-IO 会话仪表板** — 实时代理会话状态、token/成本/延迟仪表。 |
| Trendshift | **context-mode** | MCP 服务器作为协议层。钩子路由注入跨 17 平台。SQLite 会话状态。 | **NT-IO MCP 集成** — NeoTrix 通过 MCP 暴露能力。 |

---

## 二、当前架构缺口分析

| 领域 | 现有状态 | 缺口 | 优先级 |
|------|---------|------|--------|
| **KV Cache** | 配置模型 (307 行) + 元数据块管理器 (359 行) | 无运行时分配；无 GPU 集成；无 3 级分层 | P0 |
| **GWT 路由** | 10 头路由 + 5 路由策略 (960 行) | 无广播机制；无权重学习；无预算约束；无 drift 检测 | P0 |
| **NT-MEMORY** | 丰富 KB 实现 (70+ 文件) | 无 L1-L4 层级；无 compaction；无地址化存储 | P0 |
| **SEAL Pipeline** | 核心进化循环 (350 行) | 无编辑执行；无蒸馏；无技能结晶；两处断连 | P1 |
| **NT-IO/Gateway** | 生产级网关 (1290+ 行) | 无 MCP 服务器；model_routing route() 是 stub | P1 |
| **AddressableStore** | **不存在** | 需从零实现 | P1 |
| **ContextSandbox** | **不存在** | 需从零实现 | P1 |
| **ProceduralGraph** | **不存在** | 需从零实现 | P1 |
| **DynamicParams** | 纯数据模型 (328 行 x2) | 重复；无渲染集成；无时序调度 | P2 |

---

## 三、核心路线任务清单

### Phase 0: 基础修复 (1-2 天)

| # | 任务 | 文件 | 说明 |
|---|------|------|------|
| 0.1 | **cargo check 0 errors** | `neotrix-core/src/` | 修复所有 Rust 编译错误，确保 `cargo check --lib` 通过 |
| 0.2 | **前端 build 通过** | `neocodex-frontend/` | 确保 `npx vite build` 无错误 |
| 0.3 | **Tauri 启动测试** | `src-tauri/` | 确保桌面端可启动、域名注册完整 |
| 0.4 | **DynamicParams 去重** | `core/nt_core_self/dynamic_params.rs` vs `l2_perception/nt_core_self/dynamic_params.rs` | 删除 l2_perception 下的副本，统一引用 |

### Phase 1: L1-L4 内存层级 (P0, 3-5 天)

| # | 任务 | 目标文件 | 说明 |
|---|------|---------|------|
| 1.1 | **AddressableStore** | `nt_memory/addressable_store.rs` (新建) | 追加只存工具观察 + 引用指针。ARC 模式：活跃视图用 `§id` 引用，按需召回。 |
| 1.2 | **ContextSandbox** | `nt_io/context_sandbox.rs` (新建) | 沙箱执行工具输出，仅摘要进入上下文。315KB→5.4KB 级别压缩。 |
| 1.3 | **Pichay L1-L4 层级** | `nt_memory/mod.rs` | L1=活跃上下文, L2=工作集(故障钉住), L3=会话历史(有损压缩), L4=跨会话 KB。 |
| 1.4 | **Deferred Compaction** | `nt_memory/compaction.rs` (新建) | 延迟 3-5 轮再压缩。用边界查询代理。Token Eviction > Attention Matching。 |
| 1.5 | **ContextPipe 五阶段** | `nt_core_self/context_assembly.rs` (新建) | Plan→Bind→Optimize→Execute→Feedback。上下文组装可审计/可重放。 |

### Phase 2: GWT 路由增强 (P0, 2-3 天)

| # | 任务 | 目标文件 | 说明 |
|---|------|---------|------|
| 2.1 | **MTRouter 历史嵌入** | `attention_head.rs` | 添加 `history_embedding` 字段。轻量 MLP (64-d) 预测 (history, model_id) → 结果。 |
| 2.2 | **Budget-Constrained Routing** | `attention_head.rs` | salience 上下文加 `budget_remaining`。预算 < 20% → 强制路由廉价模型。 |
| 2.3 | **Drift Detection** | `nt_io_provider/gateway/drift.rs` (新建) | 滚动窗口审计 provider 质量。延迟/成本/质量漂移自动切换。 |
| 2.4 | **State-Path Tool Menus** | `attention_head.rs` | 工具菜单按状态路径排序：入口→前置条件→最终动作。减少 75% 工具表面。 |
| 2.5 | **TRACE-Router Task Pinning** | `attention_head.rs` | 高置信度任务 (>90% 历史匹配) → 入口即固定模型，跳过逐步路由。 |

### Phase 3: SEAL 进化增强 (P1, 3-5 天)

| # | 任务 | 目标文件 | 说明 |
|---|------|---------|------|
| 3.1 | **ProceduralGraph** | `nt_mind/seal/procedural_graph.rs` (新建) | 技能执行图：(procedure, relation, procedure) 三元组。LLM 精炼器编辑拓扑。 |
| 3.2 | **Distillation Stage** | `nt_mind/seal/distillation.rs` (新建) | SEAL Phase-3：从轨迹蒸馏技能模板。对比失败/成功，生成可复用模式。 |
| 3.3 | **Skill Crystallization** | `nt_mind/seal/crystallization.rs` (新建) | 技能模式成功 3+ 次 → 自动生成子代理模板 + I/O 契约。 |
| 3.4 | **Continual Harness** | `nt_mind/seal/continual.rs` (新建) | `/refine` 命令：审查轨迹，应用小的有证据支持的状态更新。快照回滚。 |
| 3.5 | **统一 SEAL 入口** | `seal/mod.rs` | 合并 core SEAL + l5_cognition SEAL 到统一模块。 |

### Phase 4: MCP 服务器 + 外部集成 (P1, 2-3 天)

| # | 任务 | 目标文件 | 说明 |
|---|------|---------|------|
| 4.1 | **MCP Server** | `nt_io/mcp_server.rs` (新建) | 暴露 NT 能力为 MCP 工具：memory_search, skill_route, context_manage。 |
| 4.2 | **MCP Tool Registry** | `nt_io/mcp_tools.rs` (新建) | NT 域能力 → MCP tool schema 映射。 |
| 4.3 | **Model Routing 实装** | `nt_io_provider/model_routing.rs` | 修复 route() stub，实现真实 LLM 调用 + 流式。 |
| 4.4 | **Trendshift Scraper** | `nt_world/trendshift_scraper.rs` (新建) | 抓取 trendshift.io 排行榜数据。 |
| 4.5 | **GitHub Trending Scraper** | `nt_world/github_trending.rs` (新建) | 抓取 GitHub trending AI/dev-tools 项目。 |
| 4.6 | **arxiv Scraper** | `nt_world/arxiv_scraper.rs` (新建) | 抓取 arxiv 最新 AI/agent 论文摘要。 |

### Phase 5: 桌面端 UX 完善 (P2, 2-3 天)

| # | 任务 | 目标文件 | 说明 |
|---|------|---------|------|
| 5.1 | **会话仪表板** | `routes/Insights.tsx` | 实时代理状态、token/成本/延迟仪表。接线后端 telemetry。 |
| 5.2 | **Settings 持久化** | `stores/settings.ts` (新建) | 密度/主题/字体等偏好 → 后端 `app_state` 持久化。 |
| 5.3 | **TaskList 同步** | `components/TaskList.tsx` | TODO checkbox → 后端 KB 持久化（非 localStorage）。 |
| 5.4 | **ChatShellProto 清理** | `routes/ChatShellProto.tsx` | 标记为 prototype，移除或降级。生产路径走 Chat.tsx。 |

### Phase 6: 架构对齐 + 冗余清理 (P2, 1-2 天)

| # | 任务 | 说明 |
|---|------|------|
| 6.1 | **删除 adapter.ts 残留** | 清理 dist 中 adapter chunk，确认无动态 import 残留 |
| 6.2 | **DynamicParams 统一** | 删除 l2_perception 副本，统一引用 core 版本 |
| 6.3 | **SEAL 统一** | 合并两处 SEAL 实现 |
| 6.4 | **SelfModel 三型文档化** | `nt_core_meta::SelfModel` vs `nt_core_self::SelfModel` vs `nt_core_self_model::SelfModel` 明确区分 |
| 6.5 | **api/index.ts 清理** | 移除不必要的 re-export，按 domain 分组 |

---

## 四、最有解架构设计

### 4.1 三层记忆架构 (对标 KVMem + Pichay + ARC)

```
┌─────────────────────────────────────────────┐
│ L1: Active Context (模型上下文窗口)           │
│   - 当前 prompt + 工具调用结果               │
│   - ContextSandbox 沙箱执行，仅摘要入窗口     │
│   - AddressableStore 引用替换旧观察           │
├─────────────────────────────────────────────┤
│ L2: Working Set (需求分页，故障钉住)          │
│   - 被重新请求的内容 → 提升到 L2             │
│   - 3-5 轮延迟 compaction                   │
│   - Token Eviction 策略                      │
├─────────────────────────────────────────────┤
│ L3: Session History (有损压缩，声明损失)      │
│   - CMV DAG 模型：快照/分支/剪枝             │
│   - 结构无损：剥离原始输出，保留合成          │
│   - 版本控制快照跨会话                       │
├─────────────────────────────────────────────┤
│ L4: Cross-Session KB (持久化)               │
│   - experience-tree 五阶段吸收               │
│   - SQLite + FTS5 + 向量索引                 │
│   - NTX 单文件可移植格式                     │
└─────────────────────────────────────────────┘
```

### 4.2 路由架构 (对标 ProgRouter + MTRouter + BAAR)

```
┌─────────────────────────────────────────────┐
│ GWT Attention Manager                        │
│   ├── Salience Scoring (phi + coherence +    │
│   │   task_progress + budget_remaining)      │
│   ├── MTRouter Embedding (history × model)   │
│   ├── TRACE-Router Pinning (高置信度固定)     │
│   ├── Drift Detection (provider 漂移审计)    │
│   └── Budget Enforcement (预算约束解码)       │
├─────────────────────────────────────────────┤
│ State-Path Tool Menu                         │
│   ├── Entry Tool (入口)                      │
│   ├── Prerequisites (前置条件，有序)          │
│   └── Final Action (最终动作)                │
├─────────────────────────────────────────────┤
│ Provider Pool                                │
│   ├── Free First (免费优先)                  │
│   ├── Circuit Breaker (熔断器)               │
│   ├── Anomaly Detector (异常检测)            │
│   └── Self-Healing (自愈恢复)               │
└─────────────────────────────────────────────┘
```

### 4.3 技能架构 (对标 Agentic Skills + Procedural Graphs)

```
┌─────────────────────────────────────────────┐
│ 9-Stage Skill Lifecycle                      │
│   1. Discovery (自主发现)                    │
│   2. Authoring (创作，SKILL.md <200 行)      │
│   3. Storage (KB 持久化)                     │
│   4. Retrieval (FTS5 + BM25 检索)            │
│   5. Routing (GWT 路由 + 子代理决策)         │
│   6. Composition (技能链组合)                │
│   7. Execution (ProceduralGraph 执行)        │
│   8. Adaptation (自适应精炼)                 │
│   9. Evaluation ( SEAL 评估)                │
│   10. Security (沙箱 + 信任分级)             │
├─────────────────────────────────────────────┤
│ ProceduralGraph                              │
│   ├── (procedure, relation, procedure)       │
│   ├── LLM Refiner: 失败/成功对比编辑拓扑     │
│   └── 保留被拒绝编辑作为反模式               │
├─────────────────────────────────────────────┤
│ Skill Crystallization                        │
│   ├── 成功 3+ 次 → 自动生成子代理模板        │
│   ├── 清晰 I/O 契约                         │
│   └── SKILL-SPEC.md 标准化                   │
└─────────────────────────────────────────────┘
```

---

## 五、执行优先级

```
Week 1: Phase 0 (基础修复) + Phase 1.1-1.2 (AddressableStore + ContextSandbox)
Week 2: Phase 1.3-1.5 (L1-L4 + Compaction + ContextPipe)
Week 3: Phase 2 (GWT 路由增强)
Week 4: Phase 3 (SEAL 进化增强)
Week 5: Phase 4 (MCP + 外部抓取)
Week 6: Phase 5-6 (UX + 清理)
```

---

## 六、参考资源

| 资源 | URL | 用途 |
|------|-----|------|
| ProgRouter | arXiv 2608.25992 | 进度引导路由算法 |
| MTRouter | arXiv 2604.23530 | 历史-模型联合嵌入 |
| KVMem | arXiv 2609.04852 | Paged KV 虚拟化 |
| ContextPipe | arXiv 2609.00749 | 五阶段上下文组装 |
| Pichay | arXiv 2603.09023 | L1-L4 需求分页 |
| ARC | arXiv 2607.25066 | Addressable Recall Compaction |
| CMV | arXiv 2602.22402 | DAG 会话状态模型 |
| context-mode | GitHub 17K★ | 沙箱工具 + FTS5 会话连续性 |
| Procedural Graphs | arXiv 2609.09153 | 自进化执行结构 |
| Prime Agent | GitHub 1.4K★ | Continual Harness |
| Eve (Vercel) | GitHub 4.9K★ | 文件系统优先 agent |
| Cherry Studio | GitHub 42K★ | AI 桌面 UX 模式 |
| OmniRoute | GitHub 17.9K★ | 多 provider 路由 |
