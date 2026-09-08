# 批量吸收报告 — 15 External Sources → NeoTrix (2026-09-07)

## Phase 1: 资料收集 (Evidence-First)

| # | Source | URL | Type | Core Mechanism | Evidence |
|---|--------|-----|------|----------------|----------|
| 1 | **utopia** | github.com/deeplethe/utopia | repo | Bitemporal knowledge graph: valid_time + belief_time, forward-chaining rules, conflict resolution, Tantivy+pgvector | README: "bitemporal knowledge graph... every fact carries when it held" |
| 2 | **foremerge** | github.com/naw103/foremerge | repo | Multi-agent coordination: semantic scope claims (12 kinds), shared SQLite, hash-chained events, ChangeSet publish→verify→accept | README: "Agents keep isolated worktrees while sharing intent" |
| 3 | **neomme** | huggingface.co/blog/Hcompany/neomme | paper | 260M/800M multimodal encoder: single bidirectional Transformer, masked discrete-diffusion, 255x compression | Abstract: "0.523 nDCG@10... outperforms all <800M models" |
| 4 | **dsh-zvec-grep** | github.com/sugarforever/dsh-plugins | plugin | Hybrid search: ripgrep + BM25 + vector (zvec engine), local-first, MCP integration | README: "zg unifies ripgrep, BM25, and vector search" |
| 5 | **wechat-intel-hub** | github.com/Rion-Wu-tech/wechat-intelligence-hub | repo | Multi-source intelligence: WeChat/tech news/social → structured output | Author ecosystem: ai-daily-briefing (162★) |
| 6 | **agentic-security** | github.com/Clear-Capabilities/agentic-security | tool | 12-pillar security scan + auto-fix pipeline: synthesize→verify→apply, dollar-cost estimates | README: "12-pillar scan... every fix is re-verified before it touches disk" |
| 7 | **reverify** | github.com/2akouwu/reverify | framework | Anti-hallucination: LLM proposes → deterministic tools verify → only verified facts survive, MCP persistence | README: "Stop your AI from making things up" |
| 8 | **swe-agent** | github.com/SWE-agent/SWE-agent | agent | Autonomous issue resolution: Agent-Computer Interface (ACI), YAML config, 12.29% SWE-bench | README: "takes a GitHub issue and tries to automatically fix it" |
| 9 | **kudu** | github.com/AdventDevInc/kudu | tool | System cleaner: rule-based scanning, platform-specific scripts, malware detection, cross-platform | README: "Free, open-source system cleaner & security scanner" |
| 10 | **spotify-portal** | engineering.atspotify.com/2026/9/portal-by-spotify | article | Context routing: PreToolUse hooks block large reads, delegate to worker models, 90% token savings | Blog: "Mean bulk-read savings were around 90%" |
| 11 | **gpt-6-astra** | developers.openai.com | docs | Async tool calling, mid-turn steering, dynamic reasoning effort, misalignment monitoring | Docs: "1,050,000 context window, async tool calling, mid-turn steering" |
| 12 | **humanlayer/skills** | github.com/humanlayer/skills/plugins | plugin | `<important if>` conditional blocks for instruction relevance, plugin marketplace | README: "exploit Claude Code's system-prompt pattern" |
| 13 | **pi-shadow-mind** | github.com/liu-zhengdong/pi-shadow-mind | agent | Shadow agents: probabilistic activation, persistent responsibilities, heartbeat triggers, report_to_main | README: "probabilistic activation via heartbeat triggers" |
| 14 | **TORCH** | github.com/Encod3d-Sec/TORCH | tool | Pentest wiki: 500+ page knowledge base, autonomous workflows, MCP search, leak detection | README: "500+ page semantic-searchable wiki" |
| 15 | **openai-research** | openai.com/index/research-acceleration | article | Research metrics: 3.1 agent-days per human-day, $600-7000/day inference, 4+ concurrent sessions | Blog: "3.1 agent-workdays of effort for every workday of human labor" |

---

## Phase 2: 世界观公理 (3 Axioms)

### Axiom 1: 知识必须有时间维度 (Knowledge Has Temporal Dimensions)

**公理**: 任何知识/事实/信念都必须携带**两个时间戳**——"事实何时为真" (valid_time) 和"系统何时相信它" (belief_time)。没有时间维度的知识 = 不可信的知识。

**NeoTrix 对应公理**: **指针守恒** — AGENTS.md 是纯指引，经验只落 KB。知识漂移 = 架构腐烂。

**证据来源**:
- utopia: "every fact carries when it held and where it came from" — bitemporal KG
- neotrix已有: `nt_temporal_facts.rs` (valid_from/until + supersession chains)
- reverify: "Grounded facts and context survive resets" — temporal persistence

**违反后果**: 知识不带时间 → 决策基于过期事实 → 系统做出矛盾行为 → 信任崩溃

**吸收映射**: 强化 `nt_memory_historian::nt_temporal_facts` (增加 explicit transaction_time tracking) + 强化 `nt_memory_kb::nt_temporal_audit` (增强 supersession chain 的 conflict resolution)

---

### Axiom 2: 多 Agent 协调基于语义声明而非文件锁 (Multi-Agent Coordination via Semantic Claims)

**公理**: 多个 agent 协作时，协调机制必须基于**语义范围声明** (scope claims)，而非文件路径锁。文件锁 = 死锁风险；语义声明 = 柔性协作 + 共享上下文。

**NeoTrix 对应公理**: **Dark Forest** — 无消费者的模块必须删除。语义声明的消费者 = 其他 agent 的协调决策。

**证据来源**:
- foremerge: "Claims describe concepts, not file paths... overlap produces a warning, never a lock"
- pi-shadow-mind: Shadow agents 通过 `report_to_main` 介入，非抢占式锁
- swe-agent: ACI (Agent-Computer Interface) = agent 与环境的语义交互协议
- neoTrix已有: `nt_core_parallel::MultiAgentCoordinator` (有 TaskContract, IntentIsolator)

**违反后果**: 基于文件锁的协调 → agent 互相阻塞 → 并行度归零 → 吞吐退化到串行

**吸收映射**: 强化 `nt_core_parallel` (增加 scope-based claims + persistent agent registry + ChangeSet publish→verify→accept pattern)

---

### Axiom 3: 推理必须经受确定性验证 (Reasoning Must Pass Deterministic Verification)

**公理**: LLM 产出的任何声明/决策/修复，必须经过**确定性工具链**验证后才能生效。LLM 提案 ≠ 事实，只有经过 verify 后的才算 fact。

**NeoTrix 对应公理**: **R-P1** `#![forbid(unsafe_code)]` — 架构不可违背原则 = 推理不可违背验证。**R-P23** 检测系统必须能检测自身缺陷。

**证据来源**:
- reverify: "LLM proposes → deterministic tools verify → only verified facts survive"
- agentic-security: "synthesize_fix → verify_fix → apply_fix — every fix is re-verified"
- TORCH: "MCP search layer so Claude always checks the knowledge base before attacking"
- spotify-portal: PreToolUse hooks 作为确定性门控
- neoTrix已有: `auto_crystallizer::anti_hallucination_gate`, `goal_contract::external_grounding_check`

**违反后果**: 未经验证的推理 → 幻觉进入生产 → 安全漏洞/错误决策 → 系统崩溃

**吸收映射**: 强化 `nt_mind::auto_crystallizer` (扩展 anti_hallucination_gate 到推理阶段) + 强化 `nt_shield::nt_shield_mcp_security` (增加 synthesize→verify→apply 流水线)

---

## Phase 3: 力量体系 (Capability Mapping to Constellation)

### NT-MEMORY 域 (知识守护者)

| 能力节点 | 来源 | 现状 | 增强点 | 映射节点 |
|----------|------|------|--------|----------|
| **双向时序知识图谱** | utopia + neotrix已有 | C2 (temporal_facts) | 增加 transaction_time, forward-chaining rules, conflict resolution 3-mode | `nt_temporal_facts` + `nt_temporal_audit` |
| **多模态检索** | neomme | C1 (vector_index) | 增加 multi-modal embedding (text+image), hierarchical pooling, asymmetric quantization | `kb_vector_index` |
| **混合搜索三合一** | dsh-zvec-grep | C3 (RetrievalMatrix) | 增加 ANN index (HNSW), regex mode fusion, on-device embedding model | `nt_memory_sweep_20260815::RetrievalMatrix` |
| **情报聚合管线** | wechat-intel-hub + kudu | C1 (crawl pipeline) | 增加多源信号聚合 + structured output + 安全清理 | `nt_world_crawl` + `nt_world_search` |

### NT-SHIELD 域 (影卫)

| 能力节点 | 来源 | 现状 | 增强点 | 映射节点 |
|----------|------|------|--------|----------|
| **安全扫描+自动修复** | agentic-security | C3 (vuln_scanner) | 增加 synthesize→verify→apply 流水线, dollar-cost 估算, 12-pillar 覆盖 | `nt_shield_mcp_security` |
| **渗透测试知识库** | TORCH | C2 (pentest_swarm) | 增加 persistent wiki (500+ pages), MCP search, leak detection, autonomous workflows | `nt_shield_pentest_swarm` |
| **系统清理审计** | kudu | C1 (audit) | 增加 rule-based scanning engine, platform-specific cleanup, malware detection | `nt_shield_sentry` |

### NT-CORE 域 (E8 引导者)

| 能力节点 | 来源 | 现状 | 增强点 | 映射节点 |
|----------|------|------|--------|----------|
| **反幻觉推理验证** | reverify + gpt-6-astra | C2 (anti_hallucination_gate) | 增加 runtime inference-time verification, LLM→deterministic verify→fact pipeline | `nt_mind::auto_crystallizer` |
| **影子 agent 持久监控** | pi-shadow-mind | C2 (parallel) | 增加 persistent shadow agents, probabilistic activation, heartbeat triggers, report_to_main | `nt_core_parallel` |

### NT-ACT 域 (行动执行者)

| 能力节点 | 来源 | 现状 | 增强点 | 映射节点 |
|----------|------|------|--------|----------|
| **多 Agent 语义协调** | foremerge | C2 (MultiAgentCoordinator) | 增加 scope-based claims, ChangeSet pattern, persistent agent registry, cross-model coordination | `nt_core_parallel` |
| **自主 Issue 解决** | swe-agent | C2 (code agent) | 增加 ACI (Agent-Computer Interface), YAML-driven config, SWE-bench pattern | `nt_act::code` |

### NT-IO 域 (界面使徒)

| 能力节点 | 来源 | 现状 | 增强点 | 映射节点 |
|----------|------|------|--------|----------|
| **条件指令块** | humanlayer/skills | C1 (skill_engine) | 增加 `<important if>` conditional blocks for instruction relevance weighting | `nt_mind_skill_engine` |
| **LLM 异步工具调用** | gpt-6-astra | C2 (provider) | 增加 async tool calling, mid-turn steering, dynamic reasoning effort | `nt_io_provider::gateway` |

### NT-MIND 域 (进化工匠)

| 能力节点 | 来源 | 现状 | 增强点 | 映射节点 |
|----------|------|------|--------|----------|
| **上下文路由优化** | spotify-portal | C2 (context_window) | 增加 PreToolUse hooks, worker model delegation, bulk-read shunt, 90% token savings | `nt_core_self::context_window` + `nt_mind::reason` |

---

## Phase 4: 一致性维护 (Conflict Detection)

### 已有能力 vs 吸收来源 — 对齐检查

| 设定 | 版本 | 前后矛盾? | 处置 |
|------|------|-----------|------|
| `nt_temporal_facts` (valid_from/until) vs utopia (valid+belief time) | v现有 | ⚠️ 缺 transaction_time | **增强**: 在现有 `valid_from/until` 基础上增加 `believed_at` 字段 (单表扩展, 不新建模块) |
| `RetrievalMatrix` (hybrid_search) vs dsh-zvec-grep (zvec) | v现有 | ✅ 一致 | **增强**: 增加 ANN index 子模式 (HNSW), 不替换现有 cosine |
| `nt_core_parallel` (MultiAgentCoordinator) vs foremerge (scope claims) | v现有 | ⚠️ 缺 scope 语义 | **增强**: 在现有 TaskContract 基础上增加 scope claim 类型 (12 kinds) |
| `anti_hallucination_gate` (crystallization) vs reverify (runtime) | v现有 | ⚠️ 仅 crystallization | **增强**: 扩展 gate 到推理阶段 (不是新建模块, 是 gate 作用域扩展) |
| `nt_shield_mcp_security` vs agentic-security (12-pillar) | v现有 | ⚠️ 缺 cost estimation | **增强**: 增加 dollar-cost 估算函数 + fix pipeline (synthesize→verify→apply) |
| `nt_mind_skill_engine` vs humanlayer `<important if>` | v现有 | ✅ 一致 | **增强**: 增加 conditional block parser (前端, 不改核心) |
| `context_window` (tracking) vs spotify-portal (active shunt) | v现有 | ⚠️ 被动 vs 主动 | **增强**: 增加 PreToolUse hook 机制 (active pruning) |

### 幂等验证

| 增强动作 | 幂等性 | 验证方法 |
|----------|--------|----------|
| 增加 `believed_at` 字段 | ✅ 幂等 | `ALTER TABLE` 加字段, 重复执行报 column exists |
| 增加 ANN index 模式 | ✅ 幂等 | `if !has_hnsw { create_hnsw() }` |
| 增加 scope claim 类型 | ✅ 幂等 | enum variant 追加, 旧代码 match _ → 不变 |
| 扩展 anti_hallucination_gate | ✅ 幂等 | gate flag 翻转, 重复=false 不变 |

---

## Phase 5: 工业化生产 (Production Plan)

### 接线点 (R-P79 Gate)

| 吸收能力 | 接线到 | 接线方式 | 预期效果 |
|----------|--------|----------|----------|
| Bitemporal KG | `nt_temporal_facts::query_valid_at()` | 增加 `believed_at` 参数 | 知识查询可追溯"何时知道的" |
| Hybrid search HNSW | `RetrievalMatrix::hybrid_search()` | 增加 `use_ann: bool` 参数 | 大规模 KB 查询速度提升 10-100x |
| Scope-based claims | `MultiAgentCoordinator::submit_task()` | 增加 `scope: ScopeClaim` 参数 | 多 agent 并发不冲突 |
| Runtime anti-hallucination | `reasoning_engine::reason()` | 增加 `verify: bool` 参数 | 推理结果可信度提升 |
| Security fix pipeline | `nt_shield_mcp_security::scan()` | 增加 `auto_fix: bool` 参数 | 从"只扫描"升级到"扫描+修复" |
| Context shunt | `context_window::estimate_tokens()` | 增加 `shunt_threshold: usize` 参数 | 大文件读取自动分流到 worker model |
| Shadow agents | `nt_core_parallel::spawn()` | 增加 `persistent: bool` + heartbeat | 持续监控主 agent 健康 |

### 门禁 (Constellation Maturity)

| 能力 | 当前 C | 目标 C | 门禁 |
|------|--------|--------|------|
| Bitemporal KG | C2 | C4 | C1: 单测验证 believed_at → C2: 集成 temporal_facts+audit → C3: benchmark query latency → C4: 接入 SEAL pipeline |
| Hybrid HNSW | C3 | C4 | C1: 编译通过 → C2: 单测 recall@10 → C3: benchmark vs cosine → C4: 接入 retrieval pipeline |
| Scope claims | C2 | C4 | C1: 编译通过 → C2: 单测 claim conflict detection → C3: benchmark multi-agent throughput → C4: 接入 parallel executor |
| Runtime anti-hallucination | C2 | C4 | C1: 编译通过 → C2: 单测 verify pipeline → C3: benchmark accuracy → C4: 接入 reasoning engine |
| Security fix pipeline | C3 | C4 | C1: 编译通过 → C2: 单测 synthesize→verify → C3: benchmark fix quality → C4: 接入 MCP tools |
| Context shunt | C2 | C4 | C1: 编译通过 → C2: 单测 threshold routing → C3: benchmark token savings → C4: 接入 PreToolUse hooks |
| Shadow agents | C2 | C4 | C1: 编译通过 → C2: 单测 heartbeat activation → C3: benchmark overhead → C4: 接入 parallel executor |

### 优先级排序 (P0-P2)

| 优先级 | 能力 | 理由 |
|--------|------|------|
| **P0** | Runtime anti-hallucination | 幻觉 = 信任根基, reverify 模式证明 LLM→verify→fact 可行 |
| **P0** | Security fix pipeline | 安全扫描无修复 = theater (R-P44), agentic-security 证明 synthesize→verify→apply 可行 |
| **P1** | Scope-based claims | 多 agent 并发是 NeoTrix 核心价值, foremerge 模式可直接强化 |
| **P1** | Bitemporal KG | 知识可信度基础, utopia 模式可直接强化 nt_temporal_facts |
| **P1** | Context shunt | Token 成本 90% 节省, spotify-portal 模式可直接接入 |
| **P2** | Hybrid HNSW | 性能优化, 不影响功能, 可渐进引入 |
| **P2** | Shadow agents | 增强监控, 非核心路径 |

### 节奏设计

```
Week 1: P0 (runtime anti-hallucination + security fix pipeline)
Week 2: P1 (scope claims + bitemporal KG + context shunt)
Week 3: P2 (HNSW + shadow agents) + 全量回归 + C4 验证
```

---

## 吸收价值评估

| Source | Value | 理由 |
|--------|-------|------|
| reverify | **HIGH** | 反幻觉是信任根基, 模式可直接复用 |
| agentic-security | **HIGH** | 修复流水线是安全域核心缺口 |
| foremerge | **HIGH** | 多 agent 协调是 NeoTrix 核心差异化 |
| utopia | **HIGH** | 双时序是知识域基础设施 |
| spotify-portal | **HIGH** | 90% token 节省直接影响成本 |
| pi-shadow-mind | **MEDIUM** | 影子 agent 有价值但非核心路径 |
| swe-agent | **MEDIUM** | 自主 issue 解决有参考价值 |
| humanlayer/skills | **MEDIUM** | 条件指令块增强 skill engine |
| gpt-6-astra | **MEDIUM** | async tool calling 是 LLM 趋势 |
| neomme | **MEDIUM** | 多模态检索有长期价值 |
| dsh-zvec-grep | **MEDIUM** | 混合搜索已存在, HNSW 是渐进优化 |
| TORCH | **LOW** | 渗透知识库已有类似实现 |
| kudu | **LOW** | 系统清理非核心路径 |
| wechat-intel-hub | **LOW** | 情报聚合已有 crawl pipeline |
| openai-research | **INFO** | 指标参考, 无直接代码吸收 |
