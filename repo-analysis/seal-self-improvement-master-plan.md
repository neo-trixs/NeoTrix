# SEAL 自进化主计划：7 源深度分析 + 迭代路径

> **日期**: 2026-05-29 | **作者**: Agent 深度分析
> **范围**: 7 新源 (SIA + SPEAR + maigret + taste-skill + Understand-Anything + carboncode + LLMs-from-scratch) + 存量 3 源 (SkillOpt/MUSE-Autoskill/SkillOpt budget)
> **核心论题**: 4 篇自改进论文 (SkillOpt + SIA + SPEAR + MUSE) 构成 **"自进化代理四引擎"**，互相补充而非重叠

---

## 0. 全景：所有吸收源统一分类

| 集群 | 源 | ⭐/引 | 核心抽象 | 与 SEAL 关系 | Tier |
|------|-----|-------|---------|-------------|------|
| **SEAL 核心进化** | SkillOpt (arXiv 2605.23904) | — | 文本学习率预算 + 拒绝编辑缓冲 | 预算控制 absorb 速率 | 🟢 P0 ✅ done |
| | SIA (arXiv 2605.27276) | 113⭐ | 双通道更新：harness + weight | 扩展 absorb() 为双通道 | 🟢 P0 |
| | SPEAR (arXiv 2605.26275) | — | 代码沙箱 eval + auto-rollback + guard metric | Guardrail 系统补全 | 🟢 P0 |
| | MUSE-Autoskill (arXiv 2605.27366) | — | 技能级记忆 + 技能生命周期 | ReasoningBank 升级 | 🟢 P0 |
| **模块增强** | maigret (30.8k⭐) | 30.8k | 3000+ 站点 OSINT username search | stealth_net recon 补充 | 🟠 P1 |
| | taste-skill (27k⭐) | 27k | Anti-slop 前端品味 Agent Skill | frontend quality gate | 🟠 P1 |
| | Understand-Anything (43.4k⭐) | 43.4k | Tree-sitter+LLM → 代码知识图谱 | ReasoningBank code KG | 🟠 P1 |
| | Strix (25.7k⭐) | 25.7k | AI 安全测试 Agent (HTTP proxy/PoC/CVSS) | 安全测试补充 | 🟢 P0 ⏸ 设计就绪 |
| | Scrapling (54.7k⭐) | 54.7k | 自适应爬虫 + checkpoint/resume | crawler 增强 | 🟠 P1 |
| **知识参考** | carboncode (56⭐) | 56 | Chinese-first CLI Agent | CLI 本地化模式提取 | 🟡 P2 |
| | LLMs-from-scratch (96.2k⭐) | 96.2k | GPT 从零实现教科书 | Transformer 模式注入 | 🔵 KnowledgeOnly |

---

## 1. SEAL 核心进化四引擎 (P0 优先级)

### 1.0 现状: SEAL 当前架构

```
Task → PanoramicBrain.route() → ReasoningEngine → absorb()
                                         ↑
                                    evaluation_history + snapshot_restore
```

**已知缺陷**:
- `absorb()` 单通道: 只更新 capability_vector，不更新 agent prompt/tools
- 评估落后: 靠 internal reward (PerformanceEvaluator) 而非结构化 eval
- 无 prompt 演化: agent 的 prompt/instruction 从不变
- 无技能记忆：absorb 后无"技能层面"的抽象和持久化

### 1.1 四引擎互补关系

```
SkillOpt (已实现)
    ├── textual_learning_rate_budget → 控制 absorb 速率
    ├── rejected_edit_buffer → 记忆失败模式
    └── 接口: ReasoningBrain.absorb() 返回 bool, replenish_budget(), check_rejected_pattern()

SPEAR (待实现) —— guardrail + sandbox
    ├── Python sandbox 执行 eval DataFrame 分析
    ├── auto-rollback on metric regression (比 snapshot_restore 更细粒度)
    ├── guard metric floor (最低性能门限，跌破自动阻断)
    └── 接口: code_eval_sandbox(), guard_metric_floor(), auto_rollback_guard()

SIA (待实现) —— 双通道更新
    ├── harness 更新: agent prompt, tools, retry logic (当前模型从不更新)
    ├── weight 更新: capability_vector (已有 absorb())
    └── 接口: update_harness(prompt_diff), dual_channel_absorb(harness_delta, weight_delta)

MUSE-Autoskill (待实现) —— 技能级记忆
    ├── skill lifecycle: 新生 → 熟练 → 固化 → 衰退
    ├── skill-level memory: 跨任务的模式抽象，非单条记忆
    └── 接口: SkillCrystallizer.crystallize(), ReasoningBank.skill_recall()
```

**集成架构**:

```
Task → SPEAR.sandbox_eval() → SIA.dual_channel_absorb()
                                   ├── harness_update (prompt/tools)
                                   └── weight_update (capability)
                                             ↓
                                   SkillOpt.budget_check()
                                             ↓
                                   MUSE.skill_crystallize()
                                             ↓
                                   SPEAR.auto_rollback_guard() [if regress]
```

### 1.2 SPEAR 深度分析

**论文核心贡献**:
1. **CodeAct 范式迁移到 APE**: 优化器不再是固定 pipeline，而是自由 agent，自主决定何时 evaluate/python/set_prompt/finish
2. **Python sandbox**: 优化器自己写 Python 代码分析 eval DataFrame（confusion matrix、error clustering、per-group metrics）— LLM 无法从原始 eval DataFrame 可靠提取 class-pair confusion
3. **auto-rollback**: 指标回归时自动回滚到上一版 prompt
4. **guard metric floor**: 可选，主指标跌到 floor 以下立即阻断

**论文结果**:
- 工业 LLM-as-judge: κ 0.857 vs 0.359 (基线)
- BBH-7: 0.938 vs GEPA 0.628 vs TextGrad 0.484
- Python tool ablation: Δ = +0.79κ (最大单个杠杆)

**→ NeoTrix 集成点**:

| SPEAR 概念 | NeoTrix 已有 | 差距 | 实现方案 |
|-----------|-------------|------|---------|
| evaluate tool | PerformanceEvaluator | 只有简单评分，无 DataFrame 级结构化 eval | 新 `evaluation/sandbox.rs`: 支持任意 eval DataFrame + Python sandbox |
| python tool | 无 | 无代码沙箱 | 新 `mcp_tools/python_sandbox.rs`: 隔离 sandbox 执行 Python 分析代码 |
| set_prompt tool | 无 | 无 prompt 版本管理 | 新 `prompt_registry.rs`: prompt 版本 + diff + rollback |
| auto-rollback | snapshot_restore | 只回滚 capability，不回滚 prompt | 扩展 rollback 到 prompt_registry |
| guard metric floor | 无 | 无条件阻断 | 新 `guard_metric.rs`: 配置 floor + 触发阻断 + 通知 |

### 1.3 SIA 深度分析

**论文核心贡献**:
1. **Harness + Weight 双更新**: harness=agent 工具/prompt/retry logic；weight=模型权重。之前两个领域互不交叉
2. **MetaAgent + TargetAgent + FeedbackAgent 三体架构**: 不是单一体，是三个 agent 协作
3. **结果**: LawBench 25.1%↑、GPU kernel 14x 加速、scRNA 20.4%↑

**→ NeoTrix 集成点**:

| SIA 概念 | NeoTrix 已有 | 差距 | 实现方案 |
|---------|-------------|------|---------|
| harness 更新 | 无 | absorb() 只更新 capability，不更新 agent prompt/tools | 新 `prompt_registry.rs`: prompt 模板 + 版本控制 |
| weight 更新 | absorb() + CapabilityVector | 对齐中 | 已有，需支持双通道协动 |
| MetaAgent | SelfIteratingBrain | 不存在生成 TargetAgent | SelfIteratingBrain 可扩展出 generate_target_agent() 方法 |
| FeedbackAgent | AbsorbValidator | 已有验证，但无逐代 improvement.md | 扩展 AbsorbValidator 生成 improvement_diff |
| improvement.md | evaluation_history | evaluation_history 只有分数，无文本洞察 | 扩展 EvaluationRecord 添加 text_feedback 字段 |

### 1.4 四引擎依赖关系

```
Phase 0 (Done)
  └── SkillOpt: learning_rate_budget + rejected_edit_buffer ✅

Phase 1a (SPEAR first — 因为 guardrail 是安全基础)
  ├── guard_metric_floor
  ├── auto_rollback_guard (扩展 snapshot_restore)
  └── prompt_registry

Phase 1b (SIA second — harness 更新依赖 prompt_registry)
  ├── harness_update (prompt_registry 已存在)
  ├── weight_update (已有 absorb())
  └── feedback_agent (improvement.md generation)

Phase 1c (MUSE third — 依赖 SIA 的双通道稳定)
  ├── skill_crystallizer
  ├── skill_lifecycle
  └── ReasoningBank upgrade

No circular dependency. Safe to implement in order.
```

---

## 2. 模块增强 (P1)

### 2.1 maigret → stealth_net/recon

| 模块 | maigret 能力 | NeoTrix 集成点 |
|------|------------|---------------|
| username_search | 3000+ 站点用户名搜索 | 新 `stealth_net/recon/username_search.rs` |
| site_db | 站点特征数据库 | 新 `data/site_db.rs` — 可缓存更新 |
| recursive_search | 从一个 ID 递归发现更多 | 扩展 `stealth_net/discovery.rs` |
| proxy/tor/bypass | 代理 + Tor + FlareSolverr | 已有的 `stealth_net/proxy_chain.rs` 可复用 |
| AI analysis | LLM 摘要 | 已有的 `reasoning_engine` 可复用 |

**Gate**: maigret 的 3000+ 站点数据库是重资产。建议只注册 KnowledgeSource，实现时用 maigret 作为外部 CLI 调用 (subprocess) 而非完整移植。类比：`maigret --json username` → parse JSON → 存入 ReasoningBank

### 2.2 taste-skill → frontend quality gate

| 概念 | taste-skill 实现 | NeoTrix 集成点 |
|------|----------------|---------------|
| SKILL.md 格式 | npx skills add 分发 | 借鉴到 NeoTrix KnowledgeSource 注册流程 |
| DESIGN_VARIANCE/MOTION/DENSITY | 1-10 旋钮 | 新 CapabilityVector 扩展轴: typography_taste, motion_taste, density_taste |
| anti-slop rules | 禁止 em-dash/generic gradients | 注册为新 KnowledgeSource::TasteQuality |

**Gate**: 不实现完整 skill 系统，只注册 TasteQuality KnowledgeSource + 3 个 taste 维度。SKILL.md 分发格式可借鉴但非 P0。

### 2.3 Understand-Anything → ReasoningBank code KG

| 概念 | UA 实现 | NeoTrix 集成点 |
|------|--------|---------------|
| Tree-sitter 静态解析 | 确定性 AST → importMap → nodes/edges | 新 `code_analysis/tree_sitter.rs` (用 tree-sitter Rust binding) |
| LLM 语义层 | 从解析结构生成摘要/tags/layer | 已有 ReasoningEngine 可复用 |
| 5-agent pipeline | project-scanner/file-analyzer/arch-analyzer/tour-builder/graph-reviewer | 已有 orchestrator + ParallelExecutor 可编排 |
| diff impact analysis | git diff → 影响传播图 | 新 `code_analysis/diff_impact.rs` |
| 多平台插件 | Claude/Codex/Cursor/Gemini 插件 | 借鉴插件分发机制 |

**Gate**: 核心价值在 Tree-sitter 确定性解析 + LLM 语义的混合 pipeline。建议注册 KnowledgeSource::UnderstandCode。Tree-sitter 解析可作为独立的 Rust crate。

---

## 3. 知识参考 (P2/KnowledgeOnly)

### 3.1 carboncode → Chinese-first CLI

**提取模式**:
- Chinese-first CLI copy (`carboncode` vs `ccode` vs `cc` 命名冲突处理)
- AGENTS.md + CARBON.md 双文件规则系统
- DeepSeek V4 flash/pro 模型预设切换

**注入**: 3 条 ReasoningMemory 种子知识

### 3.2 LLMs-from-scratch → transformer 模式

**提取模式**:
- LayerNorm vs RMSNorm 位置比较
- KV-cache 实现变体 (GQA/MLA/SWA)
- MoE 专家路由实现
- RoPE 位置编码数值稳定性

**注入**: 5 条 ReasoningMemory 种子知识 + 1 个 KnowledgeSource::LLMArchitecture (extension: transformer_patterns)

---

## 4. 完整迭代 TODO

### 🟢 P0 — SEAL 四引擎 (按依赖顺序)

| 阶段 | ID | 任务 | 文件 | 依赖 | 预计 |
|------|-----|------|------|------|------|
| **Phase 0** | P0-S22 | ✅ SkillOpt: learning_rate_budget + rejected_edit_buffer | `brain_impl.rs`, `core.rs`, `seal_loop.rs` | — | ✅ Done |
| **Phase 1a** | P0-SPEAR-1 | SPEAR: guard_metric_floor — 配置 + 触发 + 阻断 | 新 `reasoning_brain/guard_metric.rs` | — | 2h |
| | P0-SPEAR-2 | SPEAR: auto_rollback_guard — 扩展 snapshot_restore 支持 prompt 回滚 | `seal_loop.rs` + `guard_metric.rs` | P0-SPEAR-1 | 1h |
| | P0-SPEAR-3 | SPEAR: prompt_registry — prompt 版本管理 + diff + rollback | 新 `reasoning_brain/prompt_registry.rs` | — | 2h |
| | P0-SPEAR-4 | SPEAR: python_sandbox — 隔离 sandbox 执行 eval DataFrame | 新 `mcp_tools/python_sandbox.rs` | — | 3h |
| **Phase 1b** | P0-SIA-1 | SIA: harness_update — prompt_registry 集成到 absorb() | `brain_impl.rs` + `prompt_registry.rs` | P0-SPEAR-3 | 2h |
| | P0-SIA-2 | SIA: dual_channel_absorb — 同时更新 prompt + capability | `brain_impl.rs` | P0-SIA-1 | 2h |
| | P0-SIA-3 | SIA: feedback_agent — improvement_diff 生成 + 持久化 | `loop_impl/seal_loop.rs` | P0-SIA-2 | 2h |
| **Phase 1c** | P0-MUSE-1 | MUSE: skill_crystallizer — 跨任务模式提取 | `self_iterating/skill_crystallizer.rs` | P0-SIA-2 | 3h |
| | P0-MUSE-2 | MUSE: skill_lifecycle — 新生/熟练/固化/衰退 | `self_iterating/skill_lifecycle.rs` | P0-MUSE-1 | 2h |
| | P0-MUSE-3 | MUSE: ReasonBank upgrade — skill_recall() | `memory.rs` | P0-MUSE-2 | 2h |
| | P0-KS | KS/融合文档: SIA + SPEAR + MUSE + Strix | KnowledgeSource 注册 + 4 设计文档 | — | 2h |

**注意**: P0-SPEAR-1/2 与已有 SkillOpt budget 系统直接衔接——guard metric floor 是 budget 耗尽后的第二道防线。

### 🟠 P1 — 模块增强

| ID | 任务 | 预计 |
|-----|------|------|
| P1-maigret-1 | KnowledgeSource::MaigretOSINT 注册 + 融合设计文档 | 1h |
| P1-maigret-2 | `stealth_net/recon/username_search.rs` — maigret CLI 封装 | 3h |
| P1-taste-1 | KnowledgeSource::TasteQuality 注册 + 3 taste 维度注入 | 1h |
| P1-taste-2 | frontend_generation taste 层 — absorb TasteQuality 影响 frontend 输出 | 2h |
| P1-UA-1 | KnowledgeSource::UnderstandCode 注册 + 融合设计文档 | 1h |
| P1-UA-2 | `code_analysis/tree_sitter.rs` — Tree-sitter 确定性解析 | 4h |
| P1-UA-3 | `code_analysis/diff_impact.rs` — diff 影响传播图 | 3h |
| P1-Strix-1 | P0-S01: HTTP 代理引擎 (design ready) | 3-4h |
| P1-Strix-2 | P0-S02: PoC 验证引擎 (design ready) | 2-3h |
| P1-Strix-3 | P0-S03: KnowledgeSource::SecurityAttacks (design ready) | 1h |
| P1-Scrapling-1 | 融合设计文档 | 2h |
| P1-Scrapling-2 | adaptive parsing | 3-4h |

### 🟡 P2 — 远期

| ID | 任务 |
|-----|------|
| P2-carboncode | Chinese-first CLI 本地化模式提取 |
| P2-Checkpoint | Pipeline checkpoint/resume (from Claude Workflows) |
| P2-AdvVerif | Adversarial Verification (from Claude Workflows) |

### 🔵 KnowledgeOnly

| ID | 任务 |
|-----|------|
| KS-LLMArch | LLMs-from-scratch → transformer patterns → ReasoningBank (5 seeds) |
| KS-CarbonCode | carboncode → CLI patterns → ReasoningBank (3 seeds) |

---

## 5. 依赖图

```
SPEAR.guard_metric_floor ─────────────────────────────────────┐
SPEAR.auto_rollback_guard ────► SIA.harness_update ──────────┤
SPEAR.prompt_registry  ───────► SIA.dual_channel_absorb ────► MUSE.skill_crystallizer
                                     │                              │
SkillOpt.budget (done) ──────────────┘                              │
                                                                     ▼
                                                              MUSE.skill_lifecycle
                                                                     │
                                                                     ▼
                                                              MUSE.skill_recall()
```

**关键路径**: SPEAR prompt_registry → SIA harness_update → SIA dual_channel_absorb → MUSE skill_crystallizer。前面完成之前后面的无法开始。

---

## 6. 当前待办 (按优先级)

| 优先级 | 下一步 | 预计 |
|--------|-------|------|
| P0 | 写 SPEAR + SIA 融合设计文档 | 2h |
| P0 | 注册 KnowledgeSource 7 变体 (dual crate) | 1h |
| P0 | P0-SPEAR-1: guard_metric_floor 实现 | 2h |
| P0 | P0-SPEAR-2: auto_rollback_guard | 1h |
| P0 | P0-SPEAR-3: prompt_registry | 2h |
| P0 | P0-SIA-1: harness_update | 2h |
| P1 | P1-maigret-1: KnowledgeSource::MaigretOSINT | 1h |
| P1 | P1-taste-1: KnowledgeSource::TasteQuality | 1h |
| P1 | P1-UA-1: KnowledgeSource::UnderstandCode | 1h |
| P1 | P1-Strix-3: KnowledgeSource::SecurityAttacks | 1h |
