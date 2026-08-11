# ADR: NeoTrix 统一能力网络（UCN）整合决策

**Status**: Accepted
**Date**: 2026-08-11
**Context**: 整合四大资产层为一套能力网络，移除对 opencode 的运行时依赖，打造 NeoTrix 自有的能力网络。

## Context

NeoTrix 当前存在四大资产层，其中三套为自有能力体系，一套为外部运行时依赖：

```
① Skills 库 (~/.agents/skills/)          ② 意识体星系 (star-memory)
   L0-L3 四层: 13 L2 自有 + 36 L3 厂商        53星辰/11星域/173共鸣通道/289分支
   SKILL.md + profile.yaml + experience/      KB: consciousness:router / star_identities
   自进化协议 v2.2, C0-C5 成熟度             domain_nt_core/world/act/... 已在用 NT 命名

③ NeoTrix Rust 运行时 (neotrix-core)     ④ opencode 集成层 (依赖, 待移除)
   SkillEngine (T3 多处接线)                 .opencode/agents/ 10 个 NT-* agent
   capability_tree DAG 引擎 (T3)             opencode.json + @opencode-ai/plugin
   skill_crystal + auto_crystallizer (T3)    AGENTS.md/CONTEXT.md 指令加载
   SmartRouter (T3) galaxy_hygiene (T3)
   SEAL pipeline (~30 stage)  experience CLI
   skills_index 表 (KB 迁移索引)
```

### 三大核心矛盾（NT-WORLD 盘点实证, 2026-08-11）

1. **双技能索引孤岛**: `SkillEngine`（运行时扫描 YAML frontmatter）与 `skills_index` 表（KB 迁移索引）**零交叉**——`skill_search` 全仓无生产读取方（T1 级孤岛，只写不读）。
2. **三套能力网并存**: `ffi/skill_tree.rs`（iOS 内存态硬编码）vs `nt_core_capability_tree/`（petgraph DAG + EvolutionEngine 真引擎）vs `skill_crystal.rs`（运行时结晶）——各自为政，违反 R-P42（禁止平行适配器模块）。
3. **双命名体系**: NT-* 域（7+4）vs skills 域（rev/dev/des/res/sg/…）——但意识体星系已使用 `domain_nt_*` 命名（收敛方向已有共识）。

### opencode 依赖实为浅根（5 处残留，均不伤核心结构）

| 位置 | 性质 |
|------|------|
| `entry/mod.rs:86-123` | provider wizard 默认项 "opencode" |
| `nt_io_neocodex.rs:785` | SubagentDispatch 子进程调用 `opencode exec` |
| `session_manager.rs:37` | 默认 agent 命令 |
| `nt_shield/guard.rs:203` | `.config/opencode` 路径扫描 |
| `core/nt_core_axiom_tree.rs:152` | agents-guard.js 门禁注释引用（插件已移除） |

真正的依赖是**运行时**三件事：agent 执行、技能加载、工具路由。这三件 NeoTrix 已具备骨架：SkillEngine（加载）、SmartRouter（路由）、NT-IO 原生 LLM 通道（openai/anthropic/xiaohuxing，执行）。

## Decision

### 方案 A：渐进收敛（Accepted）

以 **KB 为运行时单一事实源**，分 4 阶段把四层资产收敛为一套统一能力网络（UCN）。

```
┌─────────────────────────────────────────────────────────┐
│  UCN — Unified Capability Network（单一事实源 = KB）      │
├─────────────────────────────────────────────────────────┤
│ L0 宪法层     RULES / SELF-EVOLVE-PROTOCOL / CTREE       │
│ L1 域层       NT-CORE WORLD ACT MIND MEMORY IO SHIELD    │
│               META REPAIR GOVERNANCE NEXUS SCOUT          │
│ L2 星辰层     53 星辰（能力节点 = skills 库技能）           │
│ L3 分支层     289 能力分支（SKILL.md 内容本体）             │
│ L4 运行时     neotrix CLI + SmartRouter + SkillEngine +   │
│               capability_tree + SEAL + 原生 LLM 通道      │
└─────────────────────────────────────────────────────────┘
```

### 关键决策（D1-D5）

| # | 决策 | 方案 | 理由 |
|---|------|------|------|
| D1 | 事实源 | **KB 为运行时事实源，文件系统为编辑源** | 经验/路由/状态本就在 KB；文件像 git 工作区 |
| D2 | 命名统一 | 全部收编 NT-* 域，skills 域映射为域内星辰 | CONTEXT.md 已是共享语言；`domain_nt_*` 已存在 |
| D3 | 双索引 | SkillEngine 为加载器，`skills_index` 表为其持久化缓存（写通读通） | 消除孤岛，一处索引一处加载 |
| D4 | 三套能力网 | ffi=声明层（读 DAG）→ capability_tree=引擎层（唯一可写）→ skill_crystal=产物层 | 职责分离，禁平行实现（R-P42） |
| D5 | agent 运行时 | SmartRouter 替代 task 路由；SubagentDispatch 换原生 HTTP 通道 | NT-IO 已有原生通道，非造轮子 |

### skills 域 → NT-* 域映射（收编对照）

| skills 域 | → NT-* 域 | 星名 |
|-----------|-----------|------|
| rev/officer | NT-SHIELD | Rev-明 |
| dev/implementer | NT-ACT | Dev-匠 |
| des/architect | NT-CORE | Des-观 |
| res/scholar + methodology/researcher | NT-MIND | Res-深 |
| experience-tree + nexus/weaver | NT-MEMORY | Exp-藏 / Nexus-梭 |
| meta/coordinator + sg/diagnostician | NT-META | Meta-镜 / SG-诊 |
| repair/healer | NT-REPAIR | Repair-医 |
| gov/steward | NT-GOVERNANCE | Gov-衡 |
| mil/officer | NT-SCOUT | Search-觅 |
| ed/tutor | NT-IO | Edu-灯 |
| L3 厂商 36+ | 域内能力分支（只读标记） | — |

## 迁移路线图（4 阶段）

**Phase 1 — 双索引打通（最小闭环）**
- `skill_upsert` 挂进 SkillEngine 索引路径（写通）；`skill_search` 成为 SkillEngine 查询后端（读通）
- 收编 `~/.agents/skills/` 全部技能进 KB（补迁移扫描的写入方接线）
- 验证：`cargo test` + `neotrix skill list` 与 `skills_index` 一致

**Phase 2 — 命名收编（映射层）**
- KB 写入域映射表（skills 域 → NT-* 域 → 星辰），`domain_nt_*` namespace 补齐
- CONTEXT.md 增补"技能域收编"章节（走 domain-modeling skill）

**Phase 3 — 运行时替换（opencode 去依赖）**
- `SubagentDispatch` → 原生 HTTP 通道（复用 NT-IO 现有 provider 通道）
- `entry/mod.rs` provider 列表去 opencode 项；`session_manager.rs` 默认命令改原生
- `guard.rs`/`axiom_tree.rs` 引用清理；`.opencode/` 目录归档到 `_archive/`
- 验证：`neotrix exec "任务"` 端到端跑通技能加载+路由+执行

**Phase 4 — 能力网三合一 + 门禁迁移**
- `ffi/skill_tree.rs` 改读 capability_tree DAG（去硬编码）
- agents-guard 门禁 → Rust 原生（git pre-commit hook 改调 `neotrix guard`）
- step-budget 插件 → `neotrix exec` 内置预算监控
- 验证：rev-officer 全量审查 + SelfTest T1-T3

## Consequences

### 正面
- 单一事实源（KB）：路由/技能/经验/活跃度一处查询，消灭双索引孤岛
- 命名统一到 NT-*：消除双命名体系的认知税，共享语言唯一
- 移除 opencode 运行时依赖：SubagentDispatch 换原生通道，provider 自持
- 能力网单一实现（capability_tree 引擎），R-P42 合规

### 负面 / 风险
- 迁移期双写风险：Phase 1 期间 SkillEngine 写索引与 KB 写可能冲突 → 单一写者锁（Rust 侧为准）
- 厂商技能（L3）只读边界：摄入为 KB 快照后源文件更新会过期 → `last_indexed_at` + 增量扫描兜底
- 一次性摄入丢信息风险：需 dry-run + 对比校验（rev-officer 审）
- 原生 LLM 通道替换 SubagentDispatch 需端到端验证（agent 执行质量回归）

### 明确不做（Dark Forest）
- 不重写意识体星系 Python 脚本为 Rust（Phase 4 后视使用情况再定）——先保持运行
- 不删除 SKILL.md 文件系统源（编辑源保留）
- 不为"去 opencode"而自建 LLM 对话循环（复用 NT-IO 现有通道）

## Alternatives Considered

| 方案 | 结论 | 拒绝理由 |
|------|------|----------|
| **B: 大爆炸重构** | 拒绝 | 一次性重写 agent 运行时，中断开发，不可回退，风险不可控 |
| **C: 维持双轨** | 拒绝 | 违背整合意图，opencode 依赖永存，矛盾继续累积 |
| **D: 文件系统为事实源** | 拒绝 | 经验/路由/状态已在 KB，文件系统无状态层，会开第二事实源 |
