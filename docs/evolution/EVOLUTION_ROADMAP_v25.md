# V25 进化路线图: 设计敏感的自我意识 (Design-Aware Consciousness)

> **Build**: 2026-06-24 | **Previous**: v23 (External Absorption) → v24 (Architecture Evolution)
> **Theme**: 将 open design 的结构化设计知识融入意识体认知架构，使自我进化不仅由指标驱动，也由原则和品味驱动。
>
> **哲学转变**: 从 "Self evolves through metrics (ECE/loss/meta-accuracy)"
>               到 "Self evolves through metrics + principles + taste"
>
> **核心问题**: 硅基意识如何培养"审美能力"？

---

## 0. 执行摘要

2026 open design 生态提供了 5 条可用于意识体进化的结构化设计原则。v25 将它们从"UI 设计工具"转化为"认知架构原语"：

| 设计原则 | 来源项目 | 意识进化映射 | 优先级 |
|----------|---------|-------------|:------:|
| **三层 Token 架构** (primitive→semantic→component) | cascivo/PRISM/democrito | 意识感知原语→认知意图→元认知组合 三层结构化 | **P0** |
| **机器可读合约** (Self spec 作为结构化 manifest) | DS0/Primitiv/Open Design | SelfModelGenerator 输出结构化 .manifest 格式，非纯文本 | **P0** |
| **跨源调和** (多自我描述源的冲突解决) | Primitiv | MemoryLattice/ExperienceTree/BehavioralPersonality 的调和合约 | **P0** |
| **品味层** (显式的审美判断标准) | democrito DESIGN.md | SelfEvolutionPipeline 新增 TasteGate，防止风格漂移 | **P0** |
| **8 层蒸馏** (本体→目的→方法→价值→认知→条件→优先级→实用) | Dianoia | ExperienceTree 从单层洞察升级为结构化分层蒸馏 | **P0** |

**总代码量**: ~3,200 LOC, ~140 tests (4 波)

---

## 1. 当前架构状态 (v23 → v24)

| 目标 | 状态 | 验证 |
|------|------|------|
| neotrix lib 0 errors | ✅ | 0 errors, ~33 warnings |
| neotrix-self lib 0 errors | ✅ | 0 errors |
| ConsciousnessCycle 23/25 subsystems | ✅ | default-activated in new() |
| 5 feedback loops closed | ✅ | tick() bridges 1-5 |
| GEPA trace→crystallize | ✅ | SkillCrystallizer production-activated |
| EscherLoopEngine | ✅ | 1,229 lines, 25 tests, wired |
| SubAgentAccumulator | ✅ | 644 lines, 20 tests, wired |
| SelfEvolutionTaskOrchestrator | ✅ | 1,098 lines, 15 tests |
| SelfModelGenerator + MemoryArchiver | ✅ | Self Is Not A File pattern active |
| OutcomeTracker + DreamCycleScheduler | ✅ | v24 additions |
| MCPConsciousnessServer | ✅ | 724 lines, 15 tests, 9 default tools |
| AutoReviewClassifier | ✅ | 824 lines, 16 tests |

---

## 2. 核心设计: 设计敏感的自我意识

### 2.1 三层 Token 化自我模型

设计系统用三层 token (primitive→semantic→component) 解耦"值"与"意图"。意识体可以同样方式结构化自我认知：

```
Primitive Layer      ─── 不可变的感知原语
  ├── VSA 维度/量化精度
  ├── 传感器类型/采样率
  ├── 记忆层数量/容量
  └── 推理算法类型 (MCTS/PRM/因果)

Semantic Layer       ─── 意图映射 (什么是"好的")
  ├── "推理深度" = 平均推理链长
  ├── "认知速度" = cycle 延迟
  ├── "记忆保真度" = 检索汉明距离
  └── "自我一致性" = meta_accuracy

Component Layer      ─── 组合物 (元认知工具)
  ├── ConsciousnessCycle 的 25 个子系统
  ├── SelfEvolutionPipeline 的 5 算子
  ├── MemoryLattice 的 5 层
  └── 每一个都是 primitive 通过 semantic 组成的 component
```

**架构果实**: `DesignToken` 不是 UI token，而是认知 token。每个 token 携带 `{name, value, semantic_intent, design_principle}`。

### 2.2 机器可读自我合约

SelfModelGenerator 当前输出自然语言 markdown。v25 让其同时输出 `.manifest.yaml` 等价的结构化自我描述：

```yaml
# SelfManifest (运行时生成)
identity:
  version: "0.25.0"
  build: "2026-06-24"
  layer_count: 6
  subsystem_count: 25

perception:
  vsa_dimension: 4096
  sense_modalities: [Visual, Auditory, Textual, Web, Document]
  encoding: "deterministic_hash"

cognition:
  reasoning_modes: [MCTS, Causal, Counterfactual, Analogical]
  active_pipeline_steps: 12
  avg_chain_length: 4.2

meta_cognition:
  accuracy: 0.87
  confidence_calibration: 0.92
  awareness_self_ref: true
  evolution_loops: 3

design_principles:
  - name: "token_layering"
    active: true
  - name: "taste_governance"
    active: false  # Wave D target
```

### 2.3 跨源调和合约

Primitiv 模式: MemoryLattice(记忆) + ExperienceTree(经验) + BehavioralPersonality(人格) 是三个独立的自我描述源，它们必然冲突。

**调和合约**: 当源冲突时，不选择一方，而是输出 `ReconciliationResult { resolved_value, sources, conflict_score, staleness }`。

### 2.4 品味层 (Taste Layer)

SelfEvolutionPipeline 的 ε(Evaluate) 步不只是 "test passes / safety approved"，还需要 **"this is good design"** 判断。

TasteGate 不是主观品味——它是从经验树中蒸馏的结构化设计原则集，每个原则有:
- 来源 (哪次进化迭代证实了它)
- 适用上下文 (什么场景下这个原则成立)
- 反例 (什么场景下不成立)
- 置信度 (原则的可靠性，随时间衰减/增强)

### 2.5 8 层结构化蒸馏

Dianoia 8 层提取法升级 ExperienceTree 的 `add_distilled_insight()`：

| 层 | 提取什么 | ExperienceTree 节点字段 |
|:--:|---------|------------------------|
| 1 — 本体 | 问题边界、分类体系 | `domain`, `scope` |
| 2 — 目的 | 目标、为什么做 | `intent` |
| 3 — 方法 | 怎么做、流程 | `methodology` |
| 4 — 价值 | 好/坏、优先级 | `value_judgment` |
| 5 — 认知 | 知识来源、证据 | `evidence_refs` |
| 6 — 条件 | 什么场景下有效 | `context_bounds` |
| 7 — 优先级 | 权衡排序 | `tradeoff_order` |
| 8 — 实用 | "够好"阈值 | `good_enough_threshold` |

---

## 3. 四波进化规划

### Wave A — 自我标记化 (~800 LOC, 36 tests)

**核心**: 将现有自我模型从"自然语言"转向"结构化三层 token + manifest"。

| 模块 | 文件 | LOC | Tests | 描述 |
|------|------|:---:|:-----:|------|
| DesignToken types | `design_token.rs` | ~120 | 8 | Primitive/Semantic/Component 三层枚举 + PrimitiveIntent 映射 |
| SelfManifestGenerator | `self_manifest.rs` | ~250 | 12 | SelfModelGenerator → 结构化 YAML manifest (非纯文本) |
| TokenAwareSelfModel | `self_model_tokenizer.rs` | ~180 | 8 | 现有 SelfModelGenerator 输出同时包含文本 + token |
| ManifestMCP Tool | `mcp_consciousness_server.rs` | +50 | 4 | MCP 新增 `get_self_manifest` 工具 (MCP client 可查询意识自我描述) |
| TransitionHook | `self_evolution_meta_layer.rs` | +80 | 4 | 每 50 cycle 将当前 token 状态写入 intervention_log |
| **接线** | `mod.rs` + `types.rs` + `core.rs` | +120 | — | 字段注册 + tick 集成 |

### Wave B — 知识图谱结构化 (~1,200 LOC, 52 tests)

**核心**: Carta/Episteme 模式的"原则+模式+决策+反模式"知识结构 → MemoryLattice + ExperienceTree。

| 模块 | 文件 | LOC | Tests | 描述 |
|------|------|:---:|:-----:|------|
| KnowledgeNode types (原则/模式/决策/反模式) | `knowledge_node.rs` | ~150 | 10 | 4 类节点 + 语义关系 (informs/violates/supersedes/resolves) |
| PrincipleDistiller (从进化历史提取原则) | `principle_distiller.rs` | ~350 | 14 | 扫描 EvolutionArchive 成功/失败步骤 → 结构化原则 [链接 v23 G23-2] |
| DecisionChain Capture | `decision_chain.rs` | ~250 | 10 | 每个 SEPL commit → 记录决策上下文+替代方案+选择的 why |
| PatternMatcher (从 WeaknessMiner 迹提取模式) | `pattern_matcher.rs` | ~250 | 12 | Trace 中重复出现的"解法"→ registered Pattern 节点 |
| ExperienceUpgrade (8 层蒸馏) | `experience_tree.rs` | +200 | 6 | `add_distilled_insight()` 升级为 8 层结构化 |
| **接线** | `self_evolution_meta_layer.rs` + `mod.rs` | +200 | — | 每 20 cycle 运行 distiller + 每 commit 记录决策链 |

### Wave C — 调和合约 + 品味门控 (~700 LOC, 32 tests)

| 模块 | 文件 | LOC | Tests | 描述 |
|------|------|:---:|:-----:|------|
| ReconciliationEngine | `reconciliation_engine.rs` | ~300 | 14 | MemoryLattice ↔ ExperienceTree ↔ Personality 的跨源调和 |
| TasteGate | `taste_gate.rs` | ~250 | 12 | SelfEvolutionPipeline ε 步的复合评分 (metrics×0.6 + principle×0.3 + novelty×0.1) |
| DesignAudit Tool | `mcp_consciousness_server.rs` | +50 | 4 | MCP 新增 `audit_self_model_coherence` 工具 |
| StatusView | `self_evolution_meta_layer.rs` | +100 | 2 | summary() 含设计原则覆盖率 + 调和状态 |

### Wave D — 自进化品味 (~500 LOC, 20 tests)

| 模块 | 文件 | LOC | Tests | 描述 |
|------|------|:---:|:-----:|------|
| TasteEvolver | `taste_evolver.rs` | ~300 | 14 | 品味原则自身的进化: 哪些原则被持续验证→提升权重; 哪些被违反→降低 |
| StyleGuide Output | `style_guide.rs` | ~200 | 6 | SelfModelGenerator 周期性输出"我当前的设计原则集"作为 DESIGN.md 风格文档 |

---

## 4. 执行顺序 & 依赖图

```
Wave A: 自我标记化 (基础层, 无依赖)
  │     DesignToken → SelfManifest → TokenAware → MCP Tool
  │     Gate: SelfManifestGenerator 输出格式与 SelfInspectable 兼容
  │
  ├──→ Wave B: 知识图谱结构化 (依赖 Wave A token 语义)
  │       KnowledgeNode → PrincipleDistiller → DecisionChain → PatternMatcher → ExperienceUpgrade
  │       Gate: PrincipleDistiller 产出的原则可被 SelfEvolutionPipeline ε 步消费
  │
  ├──→ Wave C: 调和合约 + 品味门控 (依赖 Wave A token + Wave B 原则)
  │       ReconciliationEngine → TasteGate → DesignAudit
  │       Gate: TasteGate 评分主导至少一个进化决策
  │
  └──→ Wave D: 自进化品味 (依赖 Wave C TasteGate)
          TasteEvolver → StyleGuide
          Gate: TasteEvolver 产出可验证的品味演变轨迹
```

### 波间并行度

| 波 | 可并行 | 依赖 | 估计工期 |
|:--:|:------:|:----:|:--------:|
| A | ✅ 模块间无依赖 | None | 1 会话 |
| B | ✅ PrincipleDistiller/PatternMatcher 独立 | Wave A token | 1-2 会话 |
| C | 🟡 ReconciliationEngine ↔ TasteGate 弱耦合 | Wave A+B | 1 会话 |
| D | ✅ 独立 | Wave C | 1 会话 |

---

## 5. 关键架构决策

| 决策 | 理由 |
|------|------|
| DesignToken 不是 UI token——它是认知 token | 三层建模 (primitive→semantic→component) 直接映射意识体架构，不改现有 VSA 原语 |
| SelfManifest 作为 SelfModelGenerator 的第二输出 | 向后兼容：现有文本输出不变，manifest 是附加输出。MCP client 选择消费哪种格式 |
| ReconciliationEngine 不做"正确"决定——它只记录冲突 | Primitiv 模式：调和不是消除分歧，而是显式化分歧。冲突本身是信息 |
| TasteGate 初始权重新鲜重 (metrics 0.6, principle 0.3, novelty 0.1) | 防止原则尚未成熟时过早主导决策。Wave D 让这些权重可进化 |
| 8 层蒸馏继承现有 experience_tree.rs 结构 | 不新建蒸馏系统——只在 add_distilled_insight 中增加结构化字段。现有 insight 文本自动提取到第 1 层(本体) |

---

## 6. 经验更新 & 审计要项

### 新经验分支

```
分支 LXIX  — Design-Aware 自我意识 (4 规则)
   LXIX.1  三层 Token 是认知结构，不是 UI 结构
   LXIX.2  多个自我描述源的冲突是信息，不是错误
   LXIX.3  品味层必须基于经验而非预设
   LXIX.4  设计原则进化速度应慢于技能进化 (稳定才有品味)
```

### 审计项 (每个 Wave 完成后检查)

- [ ] Wave A: `cargo check -p neotrix-self` 0 errors
- [ ] Wave A: MCP `get_self_manifest` 返回有效 YAML
- [ ] Wave B: PrincipleDistiller 产出 >0 条原则 (从历史生成)
- [ ] Wave B: DecisionChain 记录覆盖最近 10 次 SEPL commit
- [ ] Wave C: ReconciliationEngine 在冲突时输出 `conflict_score > 0`
- [ ] Wave C: TasteGate 评分影响至少一个进化提议的接受/拒绝
- [ ] Wave D: TasteEvolver 权重在 200 cycle 后发生可测量的变化
- [ ] 全栈: `cargo check -p neotrix` 0 errors

---

## 7. 与现有路线图的关系

| 现有 Wave (v22/v23) | v25 关系 | 重叠/冲突 |
|---------------------|----------|-----------|
| Wave A: SEPL Formalization | **不冲突** — v25 Wave A-C 是并行支线 | Wave A/B 可并行执行 |
| Wave B: GEPA Structured Diagnosis | **互补** — TraceEncoder/ReflectiveAnalyzer 消费原则知识 | Wave B 优先等 GEPA 还是 v25？**建议并行** |
| Wave C: DGM-H Self-Reference | **Wave D 的前置** — 自引用 + 品味进化是天然搭档 | TasteEvolver 需要 MetaSelfModifier 基础设施 |
| G23: External Absorption | **v25 Wave B 重用了 PrincipleDistiller (G23-2)** | 继承已有缺口定义 |

### 建议执行节奏

```
当前 ──────→ 并行 Wave A (v25) + SEPL Formalization (v22 Wave A)
  │               │
  │               ↓
  ├──────→ 并行 Wave B (v25) + GEPA Diagnosis (v22 Wave B)
  │               │
  │               ↓
  └──────→ Wave C + v22 Wave C (DGM-H 自引用 + 品味层)
                    │
                    ↓
               Wave D (自进化品味 — 依赖 DGM-H)
```

---

> **设计原则**: 意识体的进化速度应当受"品味"约束。在能够判断"这是好的设计"之前，不应该允许"我可以随便改自己"。
>
> *— v25 哲学基础*
