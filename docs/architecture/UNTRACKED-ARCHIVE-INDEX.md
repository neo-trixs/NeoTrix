# 未入库文档索引（2026-09-29 抢救性归档）

> **背景**：2026-09-29 全仓测绘发现 **78 份文档躺在磁盘上但从未入库**
> （`docs/architecture/` 26 份 + `docs/plans/` 52 份，共 13,478 行）。
> ⇒ 干净检出上它们**全部消失** —— 这是知识流失风险，不是清洁问题。
>
> **本轮处置**：① 全部 `git add` 入库（内容零改动）；② 建本索引。
> **⛔ 未做删除归并** —— 逐份取证后确认它们是 **6 大主题 + 4 条演进链**，
> 不是 78 份重复。详见 §演进链。

## 为什么没有「熔炼成一份」

原提议是「78 份熔炼为 1 份」。逐份读过后**否决**，依据三条：

1. **同主题内部是演进链，不是重复** —— `engine-upgrade-research` 有 r1/r2/r3、
   `gap-audit` 有 v1/v2、`laya-rust` 有 rewrite/complete-rewrite。
   `game-architecture-evaluation-v2` 甚至自带 v1/v2 对照表 ⇒ v2 是**增量**，
   删 v1 会丢掉对照基准。
2. **31 份吸收记录各有独立出处**（哪轮吸了哪些外部源）—— 压成一段就查不到「
   某源是哪轮进的」。
3. 违反本仓教训 **R-45**：「已经不存在了」本身是有价值的考古信息，删除不可逆。

⇒ 改为「**入库 + 索引**」：可检索性反而提升（原先 78 份无人知道存在）。

## 演进链（4 组 11 份，均**不可归并**）

| 主题 | 各版本 | 实测判定 |
|---|---|---|
| 引擎升级研究 | r1(249) / r2(189) / r3(341) 行 | 三个时点的研究报告，内容互补非替代 |
| 差距清单 | v1(52) / v2(52) 行 | v2 = T+1 后审计，评分 73/100（v1 基础上 +3）；**去掉数字后内容有实质差异**，非纯数字更新 |
| 游戏架构评测 | v1(257) / v2(98) 行 | v2 内含 v1↔v2 指标对照表 ⇒ 删 v1 则对照失效 |
| Laya Rust 重写 | rewrite(418) / complete-rewrite(1353) 行 | 后者是前者的完整版，非替代 |

## 主题分布


### 外部吸收记录（31 份）

记录每轮吸了哪些外部源；`*-absorb-*` 是按主题的单源深挖

| 文档 | 行 | 主题 |
|---|---:|---|
| `ABSORPTION-GAME-SKILLS.md` | 74 | ABSORPTION-GAME-SKILLS — 游戏 skill 八项吸收（2026-09-23） |
| `ABSORPTION-ROUND14.md` | 40 | ABSORPTION-ROUND14 — 全网技术扫荡：RLTK/bracket + refs 深挖（2 |
| `ABSORPTION-ROUND15.md` | 32 | ABSORPTION-ROUND15 — 寻路完备 + 相机死区 + 可重绑输入（2026-09-23） |
| `ABSORPTION-ROUND16.md` | 60 | ABSORPTION-ROUND16 — swordgame.ai 吸收 + 旧游戏清零 + 劍嘯江湖  |
| `ABSORPTION-ROUND17.md` | 55 | ABSORPTION-ROUND17 — 引擎掏空完成 + 真音频后端（2026-09-23） |
| `ABSORPTION-ROUND18.md` | 23 | ABSORPTION-ROUND18 — 正名 abilities + 能力边界（2026-09-23） |
| `ABSORPTION-ROUND19.md` | 36 | ABSORPTION-ROUND19 — 融合重构＋金庸叙事吸收（2026-09-24） |
| `ABSORPTION-ROUND20.md` | 39 | ABSORPTION-ROUND20 — 全网扫荡熔炼：元胞洞穴＋对象池＋Celeste 下降（2026 |
| `ABSORPTION-ROUND21.md` | 42 | ABSORPTION-ROUND21 — 武功图谱＋自动战斗＋伤害管线＋改名歸墟（2026-09-24） |
| `ABSORPTION-ROUND22-ENGINE.md` | 26 | ABSORPTION-ROUND22：引擎 Behavior 层（2026-09-24） |
| `2026-09-24-agentation-absorb.md` | 30 | Agentation 吸收（2026-09-24） |
| `2026-09-24-astra-prompts-absorb.md` | 43 | awesome-astra-prompts 吸收（2026-09-24，轻量） |
| `2026-09-24-code2skill-absorb.md` | 20 | Code2Skill 吸收（2026-09-24） |
| `2026-09-24-external-absorb-batch.md` | 120 | 外部批量吸收报告（2026-09-24，40 源去重） |
| `2026-09-24-neobot-absorb-build.md` | 81 | ntos + src-tauri 吸收 → neobot 完整构建单（2026-09-24） |
| `2026-09-24-neobot-trios-absorb.md` | 58 | OpenMuse / openbot / cumora 三源吸收 → neobot 补完（2026-09 |
| `2026-09-25-claude-product-absorb.md` | 45 | muse.ai / Claude 产品吸收 → neobot 补完（2026-09-25） |
| `2026-09-25-coco-omo-absorb.md` | 23 | cocoindex-code + oh-my-openagent 吸收（2026-09-25） |
| `2026-09-25-openim-absorb.md` | 22 | OpenIM 吸收（2026-09-25）：会话能力 + UI 交互 |
| `2026-09-25-rish-absorb.md` | 55 | rish-app 吸收 → neobot 核心建议与待优化点（2026-09-25） |
| `2026-09-26-external-absorb-batch2.md` | 69 | 外部吸收 Batch2（2026-09-26，17 源）+ 缺陷 map + 核心进化建议 |

### 五实体架构 / 蓝图（4 份）

E1–E5 五个实体的统一架构；V1/V2 与两份 FUSION 是不同深度的设计稿

| 文档 | 行 | 主题 |
|---|---:|---|
| `FIVE-ENTITY-ARCHITECTURE-FUSION.md` | 377 | NeoTrix 五实体整体架构融合 |
| `FIVE-ENTITY-BLUEPRINT-V2.md` | 476 | NeoTrix 五实体统一蓝图 v2 |
| `FIVE-ENTITY-BLUEPRINT.md` | 851 | NeoTrix 五大实体蓝图 |
| `FIVE-ENTITY-FUSION.md` | 563 | NeoTrix 五实体融合架构 |

### 游戏架构与路线（5 份）

游戏三件套架构全景 + 进化路线 + neotrix-game 掏空（strangle）路线

| 文档 | 行 | 主题 |
|---|---:|---|
| `GAME-ARCHITECTURE-MAP-2026-09-24.md` | 148 | NeoTrix Game 三件套架构全景图（2026-09-24 版） |
| `GAME-ARCHITECTURE.md` | 80 | NeoTrix 游戏架构总纲（熔炼版） |
| `GAME-ROADMAP-2026-09-24.md` | 164 | NeoTrix Game 进化路线图（2026-09-24 版） |
| `MIGRATION-ENGINE-STRANGLE.md` | 63 | MIGRATION-ENGINE-STRANGLE — neotrix-game 掏空路线图（2026- |

### 引擎研究 / 评测 / 审计（9 份）

同一对象的多时点评测/审计；**带 v1/v2/r1/r2/r3 演进链**

| 文档 | 行 | 主题 |
|---|---:|---|
| `2026-09-21-engine-upgrade-research-r2.md` | 189 | NeoTrix 引擎升级研究报告 (Round 2) |
| `2026-09-21-engine-upgrade-research-r3.md` | 341 | NeoTrix 引擎升级研究报告 (Round 3) |
| `2026-09-21-engine-upgrade-research.md` | 249 | NeoTrix 引擎升级研究报告 |
| `2026-09-21-game-architecture-evaluation-v2.md` | 98 | NeoTrix 游戏架构评测报告 v2 |
| `2026-09-21-game-architecture-evaluation.md` | 257 | NeoTrix 游戏架构评测报告 |
| `2026-09-21-gap-audit-v2.md` | 52 | 差距清单 v2 — T+1 后审计（main.rs 3950 行 / 45 模块 / 15 RON /  |
| `2026-09-21-gap-audit.md` | 52 | 差距清单 — 现状审计（main.rs 3833 行 / 45 模块 / 15 RON / 108 测试 |
| `2026-09-23-prelaunch-audit.md` | 55 | 桌面 App 上线前审计（2026-09-23 融合四仓＋ntos 吸收） |

### 架构设计 RFC / 路线（8 份）

跨系统设计稿；`DESIGN-CHANNEL-DISPATCH` 1445 行是 IM 渠道调度并发化的完整设计

| 文档 | 行 | 主题 |
|---|---:|---|
| `DESIGN-CHANNEL-DISPATCH.md` | 1445 | 设计：IM 渠道调度并发化 |
| `EVOLUTION-FRAMEWORK.md` | 57 | 小世界进化框架 — 演进阶梯与晶体知识反哺 |
| `LINGEE-INSPIRED-EVOLUTION-ROADMAP.md` | 424 | NeoTrix 进化路线 — 吸收 Lingee EAOS 核心理念 |
| `NEOTRIX-FULL-ARCHITECTURE.md` | 803 | NeoTrix 完整架构拓扑 Map |
| `NT-BODY-PRESSURE-RFC-2026-09-21.md` | 56 | NT-BODY-PRESSURE RFC — 身体感：把机器负载接进意识闭环 |
| `NT-GENERATIVE-AWAKENING-RFC-2026-09-22.md` | 134 | NT 生成式推理世界模型 — 四块接线总图（RFC） |

### 世界观 / 叙事设计（4 份）

禹迹九州 / 洪荒 / 万物修行谱 / 传统文化的世界观与剧情设计

| 文档 | 行 | 主题 |
|---|---:|---|
| `2026-09-21-cultivation-codex.md` | 91 | 万物修行谱 · 三界繁衍志 |
| `2026-09-21-epic-plot.md` | 89 | 禹迹九州正史 — 完整真实剧情（五幕） |
| `2026-09-21-honghuang-governance.md` | 74 | 洪荒世界线 · 百家治理 · 生活修行志 |
| `2026-09-21-honghuang-openworld.md` | 68 | 洪荒开放世界志 — 开天辟地，各族自演，小事推大事 |
| `2026-09-21-tradition-codex.md` | 76 | 传统文化融合志 — 天文历法 · 天气四时 · 玄学中医 · 山医命相卜 |
| `2026-09-21-worldmap-story.md` | 80 | 禹迹九州 — 世界观 / 出生地 / 主线剧情设计 |

### 其他（17 份）

迁移、护栏、渲染、压力测试等单点设计与研究

| 文档 | 行 | 主题 |
|---|---:|---|
| `EVO-LEDGER.md` | 7 | EVO 迭代账本 |
| `RENDER-MIN-2026-09-24.md` | 101 | 最小资源渲染算法设计（2026-09-24） |
| `2026-09-20-agent-guardrail-architecture.md` | 345 | NeoTrix Agent Guardrail 架构升级 |
| `2026-09-20-decision-engine-gap-analysis.md` | 332 | Decision Engine 架构差距分析 |
| `2026-09-20-laya-rust-complete-rewrite.md` | 1353 | Laya 完整 Rust 重写计划 |
| `2026-09-20-laya-rust-rewrite.md` | 418 | Laya Rust 重构计划 |
| `2026-09-20-nt-lang-revised.md` | 112 | NT-LANG 修正路线 |
| `2026-09-21-cli-removal-migration.md` | 143 | CLI 目录完全移除迁移计划 |
| `2026-09-21-open-source-research-synthesis.md` | 361 | NeoTrix 开源技术资料综合研究报告 |
| `2026-09-22-agent-browser-absorption.md` | 47 | agent-browser 吸收报告（2026-09-22） |
| `2026-09-22-lingee-agents-skills.md` | 37 | Lingee Agents & Skills（2026-09-22 收割蒸馏） |
| `2026-09-22-lingee-architecture.md` | 61 | Lingee 技术架构（2026-09-22 收割蒸馏） |
| `2026-09-22-lingee-ui-ref.md` | 102 | Lingee UI 取证（2026-09-22 round2 登录态） |
| `2026-09-22-ntcode-tauri-api-design.md` | 105 | ntcode Tauri API 设计 |
| `2026-09-22-ntos-kernel.md` | 45 | ntos × 模型内核（2026-09-22 → 0.2.0 修订） |
| `2026-09-22-tui-conversation-management.md` | 520 | TUI 对话管理 + 模型切换 Implementation Plan |
| `2026-09-22-wsd-wiki-index.md` | 43 | WSD 企业 Wiki 索引（NeoTrix 挂载页） |
| `2026-09-23-three-frame-architecture.md` | 105 | 三框终局架构（侧栏 × 对话 × 设置）2026-09-23 |
| `2026-09-24-d1-deep-feature-routing.md` | 64 | D1 深特征路由 — 设计（2026-09-24，只设计不动码） |
| `2026-09-24-d2-reuse-metric.md` | 51 | D2 结晶复用率 — 设计（2026-09-24，只设计 + 基线实测） |
| `2026-09-24-d3-trajectory-distill.md` | 57 | D3 轨迹蒸馏 — 设计（2026-09-24，只设计 + Teacher-A 冒烟） |
| `2026-09-24-d4-semantic-curiosity.md` | 41 | D4 语义好奇心 — 设计（2026-09-24，只设计不动码） |
| `2026-09-24-human-inspired-refinement.md` | 72 | 人类启发细化方向（2026-09-24，L3 方法论研究） |
| `2026-09-24-mac-train-speedup.md` | 72 | Mac 训练加速方案（2026-09-24，资料+实测环境合成） |
| `2026-09-24-master-build-checklist.md` | 49 | neobot 完整实施任务清单（2026-09-24，总控） |
| `2026-09-24-master-schedule.md` | 78 | 全域吸收总排期表（2026-09-24，第二批约 300 URL 去重归并） |
| `2026-09-24-neobot-ui-optimal.md` | 182 | neobot UI 最优解 + 能力架构 map（2026-09-24，四源熔炼） |
| `2026-09-25-neobot-audit.md` | 74 | neobot 审计（2026-09-25）：全景 map + 对标最新标准 + 核心建议 |
| `2026-09-25-neobot-sole-entry-audit.md` | 96 | 审计：neobot 作为唯一入口调用 neotrix 完整能力，还差哪些（2026-09-25） |

**未归类**：0 份

## 顺带订正一处错判

本会话早前（`e6cf3654`）判定 `DESIGN-CHANNEL-DISPATCH.md`「**从未入库、永久丢失**」，
并把恢复点写进 TODO。**那是错的** —— 该文件一直在磁盘上，**1445 行、§11/§13 完好**，
只是不在 git 里。它是 `TODO.md:303` 那条 P0（IM `/stop` 兑现）的设计依据。

⇒ 教训与 R-41 同源：**「git 里没有」≠「不存在」**。断言某资产丢失前，
必须同时查 git 历史**和**磁盘。本轮入库已把它保住。

## 维护

本索引是**一次性抢救快照**。后续新增文档请直接入库，不要攒着——
本文件记录的正是「攒着」的后果。
