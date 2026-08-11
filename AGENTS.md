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
NT-IO    (界面使徒)
```

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