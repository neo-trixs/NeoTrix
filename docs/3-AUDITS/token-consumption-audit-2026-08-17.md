# Token 消耗流程节点综合审计与优化方案 — 2026-08-17

> 审计范围: NeoTrix 全链路 LLM token 消耗节点 (Gateway / AgentLoop / NeoCodex /
> TaskDispatcher / RAG-KB / 多 Agent / 桌面端 / 遥测)。
> 方法: 证据先行 (file:line) + 外部 2026 最新研究/仓库对标 (Awesome-Agent-Context-
> Compression / ContextBudget / SWE-Pruner / ACON / LLMLingua / SkillReducer /
> Anthropic context engineering / Redis LangCache)。
> 结论速览: 7 项 P0 优化 (预计省 30-60% 输入 token), 5 项 P1, 5 项 P2。

---

## 1. 外部对标 — 2026 token 优化主线 (调研来源)

| # | 来源 | 核心结论 | 对 NeoTrix 的杠杆 |
|---|------|---------|------------------|
| E1 | Anthropic context engineering (2026) | **token 优化是上下文工程问题, 不是 prompt 缩短问题**。成本主因: 臃肿上下文、闲置 tool schema、过期会话历史 (review/rework 环消耗 ~59% token) | 把 ReAct 每轮全历史重发从"成本"变成"缓存命中" |
| E2 | Awesome-Agent-Context-Compression 综述 (2026.05, preprint 2026.05.2065) | 观察压缩/轨迹压缩/记忆压缩五分类 + 压缩策略四类 (system-controlled / external-controller / agent-controlled / learned) | 给 NeoTrix 压缩管线补主动压缩 (proactive) 与策略分层 |
| E3 | The Complexity Trap (NeurIPS 2025, 2508.21433) | **简单 observation masking 与 LLM 摘要同样有效** — 工具输出不需要昂贵重写 | 验证 NeoTrix 确定性截断方向正确, 可放大使用 |
| E4 | SWE-Pruner (2601.16746) | 代码域 token 级压缩 (LLMLingua-2) 会破坏语法; **chunk 级保留/丢弃 (函数/类块) 优于 token 级** (SWE-Bench 64% vs 54%) | 代码类工具输出应整块剪裁而非逐 token |
| E5 | ContextBudget (2604.01664) / ReSum (2509.13313) | 预算感知上下文管理 + 摘要替代搜索轨迹 | 收敛到 token 预算驱动的预算管理 |
| E6 | ACON (ICLR 2026, 2510.00615) | 长程 agent 压缩策略可学习, 但成本高 | 当前确定性规则足够, 预留 learned 升级 |
| E7 | Chain of Draft (CoD) | 每步 ~5 词草稿, 保持 CoT 精度但只花 7.6% 推理 token | forecast/decide 类 reasoning 直接换 CoD |
| E8 | SkillReducer (HKUST 2026) | agent skill 描述 -48% / body -39% token 且质量不降 | 工具 schema/系统提示词瘦身 |
| E9 | Redis LangCache / semantic caching | 语义缓存最高 -73% 成本 (命中零推理) | NeoTrix `SemanticCache` 语义层已实现但未接线 |
| E10 | 观测数据: ~84% agent turn token 是 observation token (NeurIPS 2025); 40K 预算双相裁剪 (OpenCode); 单条工具结果记录时截断 (Codex 10KB); 工具输出预摘要 (Gemini CLI); JSON ~2x YAML/TSV | 工具输出是最大单一成本源 | 全链路统一"记录时截断+预算双相" |

---

## 2. Token 消耗全景图 (节点 → 成本形态 → 浪费点)

```
┌──────────────────────────── 输入端 (input token, 占账单主导) ────────────────────────────┐
│                                                                                         │
│  [Gateway] complete_with_selection ── 语义缓存(未接线) → 候选链 → provider                 │
│       │                                                                                  │
│  ┌────┴─────────────┬──────────────────┬─────────────────────┬───────────────────┐      │
│  ▼                  ▼                  ▼                     ▼                   ▼      │
│ [AgentLoop]      [NeoCodex]        [TaskDispatcher]      [RAG/KB 注入]        [桌面端]   │
│  ReAct 全历史重发   5层压缩管线         子任务×全量原文        每轮重搜无缓存       无状态单条   │
│  (P1 #1,#2)      (最佳实践标杆)       (P0 #1 放大)           (P0 #6)             (P2)     │
│  stream 不截断    tiktoken 精确        无 token 统计                                   │
│                                                                                         │
│  [system prompt][tool schema]── 每轮稳定重发 → 应利用 provider prefix caching           │
└─────────────────────────────────────────────────────────────────────────────────────────┘
┌──────────────────────────── 输出端 (output token) ───────────────────────────────────────┐
│  forecast 3候选×3重试 9 次全量 (P0 #5) | gate 采样打分 | 结果综合 max_tokens 4096        │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. 逐节点审计 (证据先行)

### 3.1 Provider Gateway — `nt_io_provider/gateway.rs`

| 位置 | 现状 | 问题 |
|------|------|------|
| L1584-1606 | 两级缓存: `SemanticCache::get_exact` (Layer-1) + LRU ResponseCache (Layer-1.5) | **语义层已实现但 Gateway 只用 exact 匹配** (`nt_io_cache.rs` L173 `get_semantic` 存在, 无调用方) → 同义不同文完全不能命中 |
| L941-963 | `prompt_cache_key` 指纹硬化 (max_tokens/thinking/tools/structured) | ✅ 正确, 保留 |
| L1627 | 每查询成本预算用 `prompt_key.len()/4` 估算 | CJK 文本被低估 4x, 与 `context_budget.rs` 口径不一致 |
| L1645 | 候选链 8 个 provider, aggressive retry | 失败放大: 每次重试同量输入 token 全付 |

**优化**:
- **P0-A1 接线语义缓存**: 命中链路加 `get_semantic` 余弦相似度层 (用 VSA HyperCube 或本地 embedding), 阈值 ~0.92。命中即零推理 (对标 E9, -73% 成本场景)。
- **P0-A2 Provider prefix caching**: ReAct 的 system+tools+稳定历史是前缀 → 按 provider 打 cache-control (Anthropic cache_control / OpenAI automatic prefix / DeepSeek context caching)。这使"每轮全历史重发"成本趋近于增量。需在 `LlmRequest` 加 `cacheable_prefix` 标记 + 消息排序保证稳定前缀在前。
- **P1-A3 成本估算口径统一**: 改用 `context_budget::estimate_tokens`。
- **P2-A4 重试上限**: 候选链 8→3, 重试仅限瞬时错误 (429/5xx), 4xx 不重试。

### 3.2 AgentLoop — `nt_io_agent_loop.rs`

| 位置 | 现状 | 问题 |
|------|------|------|
| L572 | `build_request` 克隆全历史 + `apply_context_budget` | ✅ 预算压缩在克隆上, 不污染持久历史 — 正确模式 |
| L610-616 (非流式) | 工具输出超 `max_tool_output_tokens` 时 `truncate_preserving` 60/40 | ✅ 正确 |
| L369 (stream) | **工具结果全量入历史, 无截断** | ✗ 与流式路径不对称: `turn_stream`/`turn_stream_with_approval` 工具输出全量注入, 靠下轮 build_request 兜底 → 历史残留超长输出每轮重付 |
| L736-753 | `trim_history` 按条数 (max_history=64) 裁剪 | 按条不按 token: 大消息撑爆窗口前不触发; 小消息多时浪费预算 |
| L86-89 | 默认 budget 24_000 / max_history 64 / max_tool_output 3_000 | **入口 (entry/mod.rs L2065/L2177) 未按模型 context_window 配置**, 一律默认值; 模型能力在 `nt_core_model_skills.rs` 已有注册 |
| L577-585 | `temperature:0.7, max_tokens:4096` 硬编码 | 与模型/任务类型不匹配 |

**优化**:
- **P0-B1 stream 路径补截断**: 在 `turn_stream`/`turn_stream_with_approval` 回填 Tool 消息前调用 `truncate_preserving` (复用 L610 逻辑)。工具输出是 ~84% 的 token 源 (E10)。
- **P1-B2 token 感知裁剪**: `trim_history` 改为 `estimate_messages_tokens` 驱动; 加双相: 先截断超大工具输出, 再逐轮驱逐, 最后 (90% 预算) 触发 LLM 摘要 compaction (OpenCode 40K 双相模式, E10)。
- **P1-B3 模型感知预算**: 入口按 active model 的 `context_window` × 0.8 派生 budget; 显式 `with_*` builder 参数透传。
- **P2-B4 输出端约束**: 按任务类型设 `max_tokens`; reasoning 任务用 CoD 提示 (E7) 省输出 token。

### 3.3 NeoCodex — `nt_io_neocodex.rs` (最佳实践标杆)

| 位置 | 现状 | 问题 |
|------|------|------|
| L446-453 | `count_tokens` tiktoken cl100k_base + chars/4 回退 | ✅ 精确 |
| L508-611 | 5 层压缩: budget reduce → snip → microcompact → context collapse → auto-compact | ✅ 业界领先, 保留 |
| L557 | Layer-3 microcompact 用 `chars().take(200)` | 字符口径与 tiktoken 混用; 改 `count_tokens` 对齐 |
| L2396-2401 | assistant+tool 上下文入队用 chars/4 估算 | 与 `use_tiktoken=true` 并存 → 双重口径 |
| L908 | `DEFAULT_SUBAGENT_MODEL="gpt-4o-mini"` | subagent 无父上下文传递 — 隔离好但不传压缩摘要 (信息损失, 非 token 浪费) |
| L2457-2459 | `budget_react_messages` 禁用工具截断 (靠 Layer-3) | ✅ 可接受 |

**优化**:
- **P1-C1 口径统一**: Layer-3 与入队估算全部走 `count_tokens`。
- **P2-C2 subagent 传压缩摘要**: subagent 返回时带回结构化摘要而非原始结果 (对标 sub-agent-as-context-management, E1 Pattern 5)。**✅ 已落地**: `SubagentDispatch::compress_context` + `build_request_with_context` + `run_with_context`/`run_parallel_with_context`, `dispatch_subagents` 自动注入父对话 2k 字符摘要 (nt_io_neocodex.rs)。
- **P2-C3 稳定前缀前置**: 保证 system+工具定义始终在消息最前, 最大化 prefix cache 命中 (配合 P0-A2)。

### 3.4 TaskDispatcher — `nt_core_task_dispatcher.rs`

| 位置 | 现状 | 问题 |
|------|------|------|
| L419-440 `build_sub_task_prompt` | **每个子任务 prompt 内联完整 `original_task`** | ✗ O(N×task) 放大: 3-5 子任务 × 全量原文重复注入 (P0 最严重) |
| L365-379 分解 prompt | original_task 全量入分解调用 | 合理, 仅 1 次 |
| L841-864 `build_full_prompt` | context + sub_task.prompt 全量拼接 | 无裁剪 |
| L892-913 `aggregate_results` | Reducer 去重后全量子任务结果入 LLM 综合 prompt | 结果未封顶, max_tokens=4096 |
| L699 | `tokens_used: 0` TODO | usage 未统计 |

**优化**:
- **P0-D1 子任务原文降量**: 分解后只向每个子任务传"任务摘要" (如任务标题 + 前 200 字意图 + 相关上下文切片), 或传共享引用; 完整原文仅存分解层。预期直接削减 (N-1)×task_len token/批。
- **P1-D2 结果封顶**: 每个子任务结果进入综合 prompt 前按 `estimate_tokens` 截断 (保留 60/40), 或用 reducer 输出结构化压缩。
- **P2-D3 usage 回填**: 从 Gateway `Usage` 真实聚合到 `tokens_used`。

### 3.5 RAG / KB 注入

| 位置 | 现状 | 问题 |
|------|------|------|
| `engine_core.rs` L562/L1402-1419 | 每轮推理 `kb.search(query,3)` 重搜重注入, summary 无 token 预算 | ✗ 每轮全量重付 + 无会话内缓存 |
| `engine_core.rs` L1420-1432 | `build_artifact_context` take(5) 每条截 80 字符 | ✅ 已封顶 |
| `attention_router.rs` L193-231 | title-only, limit 4 | ✅ 已最小化 |
| `nt_core_reasoning.rs` L305-354 | 检索转向量不注入文本 | ✅ 省 token |

**优化**:
- **P0-E1 KB 上下文缓存 + 预算**: 会话内按 query 缓存检索结果; 注入前 `estimate_tokens` 封顶 (如 512 token), 超限按相关度截断。对标本项目已确立的预算引擎。
- **P2-E2 去重**: 相邻轮次相同 KB 命中合并为增量差异。

### 3.6 多 Agent / 决策节点

| 位置 | 现状 | 问题 |
|------|------|------|
| `nt_core_forecast.rs` L421/L551-556 | **3候选×3重试=9 次全量调用** (作者已注释为已知放量点) | ✗ 但仅靠预算门兜底, 未降调用次数 |
| `nt_core_gate.rs` L799+ | complete × samples 采样打分 | 采样数 × 完整调用 |
| `nt_core_parallel/coordinator.rs` L203-293 | SharedContextWindow budget | ✅ 已有预算共享 |

**优化**:
- **P0-F1 forecast 放量收敛**: 候选 3→1 主 + 1 备用; 重试仅瞬时错误; 输入预算 800→可按模型窗口 × 0.05 缩放; 内部推理换 CoD 提示 (E7) 省 90%+ 输出 token。
- **P1-F2 gate 采样降本**: 采样数从固定改为"按分差早停" (连续 N 个高分即停)。

### 3.7 桌面端 / 遥测 (数据真实性审计)

| 位置 | 问题 | 性质 |
|------|------|------|
| `src-tauri/src/commands/agent_view_cmds.rs` L405 | `tokens_used += pseudo_rand_int(seed,500)` — **伪随机虚构统计** | ✗ 若为演示视图可接受, 若为生产遥测则数据失真 |
| `src-tauri/src/commands/neocodex_cmds.rs` L198-200 | `tokens_used += len/4` 字符估算 | ✗ 应取 Gateway `Usage` |
| `src-tauri/src/commands/chat_cmds.rs` L59-65 | 桌面聊天只发单条消息, 无历史传递 | 功能性缺口 (上下文全丢), 非浪费 |
| `src-tauri/src/anthropic/client.rs` L32+ | 直连 Anthropic 不经 Gateway | 双轨无预算/缓存/去重 |
| `nt_core_task_dispatcher.rs` L699 | `tokens_used:0` TODO | 未接线 |
| `nt_io_telemetry.rs` L117 | `total_prompt_tokens AtomicU64` 真实计数器 | 接线方待确认 |

**优化**:
- **P0-G1 真实 usage 贯通**: 以 Gateway `record_success` 的 `Usage` 为唯一事实源, 桌面端/视图/统计统一订阅; 删除伪随机与 len/4 估算。
- **P2-G2 桌面 chat 恢复历史**: 挂载 AgentLoop (带预算) 或传入最近 K 条消息。**✅ 已落地**: 每会话 `HISTORY` (HashMap<String, VecDeque<Message>>) + 24k token/40 条双控裁剪, 请求携带全历史并标注 prefix cache。
- **P2-G3 桌面 Anthropic 直连并入 Gateway**: 消除双轨。**✅ 已落地**: 桌面聊天改走统一 `GatewayV2` (懒构建 + key 变更重建, Anthropic provider, 响应缓存), `anthropic::client::send_message_stream` 直连路径已删除 (client.rs 只留 key 存储)。

### 3.8 跨节点 — Token 计数口径分裂 (系统性)

四处估算口径并存:
1. `context_budget.rs` L41 — CJK 感知字符估算 (CJK 1 token/char, 其他 1/4)
2. `context_strategy.rs` L47-54 — ascii/4 + non_ascii/2 (CJK 按 2, **低估 2x**)
3. `neocodex.rs` L446 — tiktoken cl100k_base (精确)
4. 多处 `len/4` — 低估 CJK 4x

**后果**: 同一段历史在不同节点被估出不同 token 数 → 有的过早驱逐 (质量损), 有的溢出 (报错)。**P0-H1 统一估算器**: 进程级 `OnceLock<tiktoken>` 单例 (neocodex 已有模式), `context_budget.rs`/`context_strategy.rs`/gateway 成本预算/桌面端全部改走它, CJK 回退保留。

---

## 4. 优化优先级汇总 (P0 → 落地顺序)

| # | 节点 | 动作 | 预期收益 | 文件 |
|---|------|------|---------|------|
| P0-1 | TaskDispatcher | 子任务只传任务摘要, 砍原文重复注入 | 省 (N-1)×task_len/批, 高频 | `nt_core_task_dispatcher.rs` |
| P0-2 | AgentLoop | stream 路径工具输出补截断 (60/40) | 84% token 源收敛 | `nt_io_agent_loop.rs` L369 |
| P0-3 | Gateway | 接线语义缓存 `get_semantic` | 同义查询零推理 | `gateway.rs` + `nt_io_cache.rs` |
| P0-4 | Gateway | provider prefix caching (稳定前缀) | ReAct 重发趋近增量成本 | `LlmRequest` + gateway |
| P0-5 | Forecast | 候选 3→2, 重试限瞬时, 输入预算缩放, CoD | 9→~2 次全量调用 | `nt_core_forecast.rs` |
| P0-6 | RAG/KB | KB 检索会话内缓存 + 注入预算封顶 | 每轮重付 → 首轮一次 | `engine_core.rs` |
| P0-7 | 口径统一 | 单一 tiktoken 估算器贯通全链 | 消除低估/高估系统性偏差 | `context_budget.rs`/`context_strategy.rs` 等 |
| P0-8 | 遥测真实化 | Usage 单一事实源, 删伪随机/len/4 | 数据可信 | `agent_view_cmds.rs`/`neocodex_cmds.rs` |

P1: AgentLoop 入口模型感知预算 (✅ P1-B3) / token 感知 trim + 双相 compaction / gate 早停 (✅ P1-F2) / 子任务结果封顶 / gateway 成本口径统一。
P2: subagent 压缩摘要 (✅ P2-C2) / 桌面 chat 恢复历史 (✅ P2-G2) / 桌面并入 Gateway (✅ P2-G3) / retry 上限 / 输出端 CoD。

---

## 5. 落地纪律

**落地状态 (2026-08-17)**: P0 全 8 项 + P1-B3 + P1-F2 + P2-C2/G2/G3 已闭环并验证 (cargo check -p neotrix / neotrix-tauri 0 error; 相关测试通过)。剩余未做: P1-B1/B2/D2 + P2-A4/B4/C3/D3/E2/G1。

- 复用既有基建: 预算引擎 (`context_budget.rs`)、tiktoken (`neocodex::count_tokens`)、语义缓存 (`nt_io_cache::SemanticCache::get_semantic`) — **不新建平行适配器 (R-P42)**。
- 改动后可观测: `BudgetResult` (original/final/tool_outputs_truncated/messages_evicted) 已是观测杠杆, 各节点应用后记录削减量。
- 验证: `cargo check --all-targets -p neotrix` + `cargo test -p neotrix --lib -- context_budget`; 结构变更后 clean build (R-P9)。
- 新术语如需入库 (如 "observation masking" / "prefix caching"), 走 `domain-modeling` 更新 CONTEXT.md。