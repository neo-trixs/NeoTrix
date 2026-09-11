# NeoTrix — AI-Native Developer Toolkit

NeoTrix is an AI-native developer toolkit with self-evolving reasoning, knowledge representation via VSA HyperCube, and Global Workspace Theory attention routing.

**Preamble**: This session loads `CONTEXT.md` (root) as the shared language prefix. All domain terms used in this project are defined there. Before using any domain term, refer to CONTEXT.md for its precise definition and avoid column.

**统一吸收协议 (MUST)**: 每次会话结束必须自动执行 `experience-tree` 五阶段吸收 (快照→蒸馏→分类→落盘→反馈)。经验统一写入 `~/.neotrix/knowledge.db` 的 `kv_store` `experience` 命名空间; **AGENTS.md 不含任何 per-cycle 增长区** — cycle 指针、摘要、正文全部只存 KB hub。执行: 汇总会话经验 → `~/.neotrix/pending-absorb.json` → **NeoTrix 自身后台循环** (`nt_mind_background_loop::handlers_absorption`, 60s tick) 自动调 `neotrix-experience absorb` + `close --cycle NNN`——不再依赖任何 opencode 插件。协议详见 `~/.agents/skills/experience-tree/SKILL.md`。

**指针守恒 (HARD RULE)**: AGENTS.md 是**纯指引文档**，永久禁止追加以下内容：cycle 完整正文、Session 明细表、 Build Baseline 明细、元认知发现清单、吸收细节、**以及任何 per-cycle 增长区（含 Experience Index 指针表）**。所有 cycle 内容（指针+摘要+全文）统一经 `experience-tree` 流程落盘 KB `experience` hub；AGENTS.md 仅允许修订操作规则（Dev Rules/审查维度/共享语言）本身。新内容超过 3 行 → 必须走 KB 吸收流程。违反即回滚。

**写入门禁 (MECHANISM)**: 经验指针与全文统一存 KB `experience` namespace hub，**AGENTS.md 禁止内联任何经验表、cycle 正文或增长区**（手工追加会被门禁拒绝）。指针检索唯一路径：`neotrix-experience hub` 查看 cycle 索引 / `query --kw` 检索全文。AGENTS.md 结构受 git pre-commit hook 保护：拒绝超阈/含索引提交。规则由机制执行，不依赖 agent 自律。

**外部文件惰性加载 (LAZY LOAD)**: 本文件是 L1 常驻层，只含最高信号内容。以下文件**不要预加载**，遇到相关任务时用 Read 按需读取，加载后为强制规则：
- `@dev-rules.md` — 全量 R-P1-R-P80 (编码/构建/审查/吸收纪律)。处理编码、构建、审查、吸收任务时加载
- `~/.agents/skills/rev/officer/rev-officer-agent.md` — D1-D51+S1-S7 审查维度。触发 review 时加载
- `~/.agents/skills/experience-tree/SKILL.md` — 吸收协议。会话收尾时加载
- KB 检索: `neotrix-experience query` — 历史经验全文，需要时按关键词查询
## Skill Routing

| 任务 | 加载 |
|------|------|
| `review`/审计/审查/盘点 | `rev-officer-agent.md` → NeoTrix Max 全量审查 (D1-D51+S1-S7) |
| 吸收外部仓库/技术 | `skills/external-absorption/SKILL.md` (C1-C6 契约) + `dev-rules.md` (R-P42/R-P79/R-P80) |
| 会话收尾 | `experience-tree/SKILL.md` → 五阶段吸收 |
| 探索代码库 | `skills/codebase-exploration` (语义搜索/依赖图) |
| 实现功能/修复 | `dev-rules.md` (编码/构建/持久化纪律) + `skills/dev/implementer` |
| 架构设计 | `skills/des/architect` |

## Architecture

```
NT-CORE  (E8引导者)  | NT-MIND  (进化工匠)  | NT-MEMORY (知识守护者)
NT-WORLD (虚空探索者) | NT-ACT   (行动执行者) | NT-SHIELD (影卫)
NT-IO    (界面使徒)   | NT-PHYSICAL (具身骨架) | NT-FEEL (情感中枢)
```

### 3-Layer Architecture (意识-具身-能力网)

```
L5 Consciousness (涌现觉知)  →  E8 + GWT + Emotion + SEAL
L3 Embodiment (具身骨架)     →  sensors + motors + safety + power
L1 Capability Network (能力网) →  tools + perception + memory + action
```

### 6-Layer Architecture (新架构)

```
L6 Meta-Cognition (元认知层)  →  nt_meta + nt_repair + nt_nexus
L5 Cognition (认知层)         →  nt_core + nt_mind
L4 Emotion (情感层)           →  nt_feel (core emotion engine)
L3 Embodiment (具身层)        →  nt_physical + nt_shield + nt_feel
L2 Perception (感知层)        →  nt_world + nt_sense
L1 Action (行动层)            →  nt_act + nt_io + nt_memory
```

### Layer Directories

```
neotrix-core/src/
├── l1_action/          # L1 行动层
│   ├── traits.rs       # ActionLayer trait
│   ├── nt_act/         # 工具/动作
│   ├── nt_io/          # IO/接口
│   └── nt_memory/      # 记忆
├── l2_perception/      # L2 感知层
│   ├── traits.rs       # PerceptionLayer trait
│   ├── nt_world/       # 世界感知
│   └── nt_sense/       # 感官处理
├── l3_embodiment/      # L3 具身层
│   ├── traits.rs       # EmbodimentLayer trait
│   ├── nt_physical/    # 身体模式
│   ├── nt_shield/      # 安全/保护
│   └── nt_feel/        # 情感具身
├── l4_emotion/         # L4 情感层
│   ├── traits.rs       # EmotionLayer trait
│   └── nt_feel/        # 情感引擎
├── l5_cognition/       # L5 认知层
│   ├── traits.rs       # CognitionLayer trait
│   ├── nt_core/        # 核心推理
│   └── nt_mind/        # 自我进化
└── l6_meta/            # L6 元认知层
    ├── traits.rs       # MetaLayer trait
    ├── nt_meta/        # 元认知协调
    ├── nt_repair/      # 自愈修复
    └── nt_nexus/       # 跨会话记忆
```

### Key Components

| Component | Purpose | Location |
|-----------|---------|----------|
| **PerceptionBridge** | Attention-gated bridge connecting SensoryIntegrationHub with SelectiveState | `l2_world_impl/nt_world_sense/perception_bridge.rs` |
| **CapabilityBridge** | Bridge connecting evolution view (CapabilityTree) with runtime view (CapabilityRegistry) | `nt_core_capability_tree/src/bridge.rs` |
| **HeartbeatAggregator** | Unified system health signal collector | `core/nt_core_heartbeat.rs` |
| **EmotionLabel** | Unified emotion enum (11 variants) | `core/nt_core_self/emotion_state.rs` |
| **ImageSuperResolver** | Image super-resolution engine with 12 models, tiled inference, auto-download | `nt_file_ability/image_super_resolution.rs` |
| **PdfIconEnhancer** | PDF icon enhancement pipeline (extract→SR→embed) | `nt_file_ability/pdf_icon_enhance.rs` |
| **XObjectImageIterator** | Generic XObject tree traversal for PDF image extraction | `nt_file_ability/pdf_image_extract.rs` |
| **BackendManager** | Auto-selects best SR backend (ONNX/Interpolation) | `nt_file_ability/image_super_resolution.rs` |
| **TiledSuperResolver** | Memory-bounded tiled inference engine | `nt_file_ability/image_super_resolution.rs` |
| **PdfEnhanceCapability** | UnifiedCapability impl routing PDF enhance via CapabilityRegistry | `nt_file_ability/capability.rs` |

### SelfModel Types (neotrix-core)

| Type | Purpose | Avoid |
|------|---------|-------|
| `nt_core_meta::SelfModel` | Static structural identity (modules/files/dependencies) | "the self model" |
| `nt_core_self::SelfModel` | Dynamic performance model (capability/uncertainty/fatigue) | "performance model" |
| `nt_core_self_model::SelfModel` | Value function model (identity/goals/weights) | "value model" |

- **技能节点 3 层**: Small Passive (微节点自愈) / Notable Passive (域级突破) / Keystone (跨域变革)
- **Ascendancy 双专精**: 每 session 两个 Weapon Set，经 `nt_core_self::AttentionManager` 按任务类型路由
- **Rune Socketing 5 槽**: Crimson(数据摄取) / Indigo(变换) / Obsidian(缓存) / Golden(错误恢复) / Alabaster(监控)；组合产生 Runeword (如 Scry = 完整 ETL)
- **Constellation 成熟度**: C0 编译 → C1 单测 → C2 集成测试 → C3 benchmark → C4 主流水线 → C5 自愈/自适应

## Always-On Core Rules

- **R-P1**: `#![forbid(unsafe_code)]` — zero unsafe in core
- **构建缓存不可信**: 结构变更后强制 `cargo clean` 或连续 build 两次获取真实错误计数 (R-P9/R-P17/R-P29/R-P35/R-P51/R-P54)
- **R-P16**: 每次编辑后 re-read 文件验证持久化 — 不信工具成功消息
- **R-P79**: 外部技术吸收必须同 session 接线到生产路径，禁止延期死代码
- **R-P42**: 吸收强化现有节点，禁止平行适配器模块
- **R-P81**: 清理前必须归档 — 使用 SafeDeleter 时默认启用 archive_before_delete，防止误删不可恢复数据
- **R-P82**: 清理风险分级 — RiskAssessor 评分 ≥60 必须人工确认，评分 ≥80 自动拒绝执行
- **R-P83**: 清理白名单优先 — RiskAssessor 白名单路径跳过风险评估，直接标记为 Safe
- **R-P84**: 清理事件日志 — CleanupCoordinator 必须记录所有清理事件到 event_log，支持审计追溯
- **全量规则**: 见 `@dev-rules.md` (处理编码/审查任务时加载)

## Shared Language

**Preamble**: Every session loads `CONTEXT.md` at the root as the shared language prefix. All domain terms used in this project are defined there. Before using any domain term, refer to CONTEXT.md for its precise definition and avoid column.

New terms are proposed via `domain-modeling` skill (`skills/engineering/domain-modeling/SKILL.md`) — scan for inconsistencies, stress-test against code, then update CONTEXT.md.

Key shared language decisions:
- Always use the `nt_` prefix for module names (e.g., `nt_core_self`, `nt_world_crawl`)
- Always use the full domain name when referring to a domain (e.g., "NT-WORLD" not "crawler", "NT-CORE" not "core")
- Distinguish "KB embedding" (vector storage) from "VSA embedding" (symbolic representation)
- Distinguish "ConsciousnessTree" (meta-cognition loop) from "GWT" (attention routing)
- Use C0-C6 constellation notation for module maturity (not "levels" or "stages")
- Use "T1/T2/T3" for SelfTest wiring tiers (not "partial/full")

## Axioms (2026-09-08, 8-Source Batch)

| # | Axiom | Implication |
|---|-------|-------------|
| A1 | **Cost-Aware Routing** — Not all tasks need the strongest model | GWT salience 加入 token 成本权重；cheap models for I/O, expensive for reasoning |
| A2 | **Context as Scarce Resource** — Context window / KV capacity is the fundamental bottleneck | KVMem paged KV for >256K sessions; compaction for <256K; adaptive switching |
| A3 | **Skill as Production Template** — Skills are structured, composable, versionable expert knowledge | SKILL-SPEC.md contract (<200 lines) for all NT-* skill implementations |

## Cross-Source Patterns (5)

| # | Pattern | Definition | NeoTrix Mapping |
|---|---------|-----------|-----------------|
| P1 | **Model Routing / Delegation** | Route tasks to cheapest capable model | GWT salience + cost weight |
| P2 | **Isolation-per-Task** | Each task gets isolated context/state | Worktree isolation + paged memory |
| P3 | **Profile-Driven Adaptation** | Persistent profile shapes behavior across sessions | SelfModel extension |
| P4 | **Ordered Backend Fallback** | Single interface with ordered fallback chain | Ordered Backend Router |
| P5 | **Skill as Reusable Template** | Skills are composable atoms with strict interfaces | SKILL-SPEC.md contract |

## Build

```sh
cargo build -p neotrix              # CLI
cargo build -p neotrix-tauri        # Desktop
cargo check --all-targets -p neotrix
cargo check --features full --lib -p neotrix
```

## Test

```sh
cargo test -p neotrix --lib         # Unit tests
cargo test -p neotrix --lib -- <test_name>
npm test                            # Frontend tests
```

## Key Locations

| Path | Purpose |
|------|---------|
| `neotrix-core/src/` | Main crate |
| `neotrix-core/src/core/` | Foundation: E8, HyperCube, GWT |
| `neotrix-core/src/neotrix/` | All subsystem modules |
| `neotrix-core/src/cli/` | CLI command definitions |
| `crates/` | Shared libraries |
| `src-tauri/` | Desktop app |