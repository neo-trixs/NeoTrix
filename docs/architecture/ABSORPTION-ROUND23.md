# ABSORPTION-ROUND23 — 代码图谱 + Skill 工程 + UI 方向（2026-09-30）

> **Absorption Date**: 2026-09-30 | **Status**: Landed (2) + Deferred (4)
> **Batch**: 用户粘贴 ~400 URL（大面积重复；先与 `absorption-sources/repos.csv` 去重，
> 主流项 mem0/cognee/graphiti/letta/browser-use/cua/crewAI/autogen/langgraph 等**已在库**）。
> 本轮只读 5 个未收录源（README 级 webfetch，不克隆不执行代码）+ 1 个本地目录源（oil-ui）。
> **拒收项声明**：用户指令中"去 GitHub 搜 OPENAI_API_KEY 批发密钥"一条**拒绝执行**
> （凭证收割，有害行为）。

## 1. Triage

| Pri | Items | Rationale |
|-----|-------|-----------|
| P0 read (5) | R23-S1–S5 below | 直接对应 NeoTrix 缺口：调用边工具链（G1/G2/G4）、Skill 工程规范、UI 方向方法论、安全评审 |
| P1 next | `Leonxlnx/taste-skill`（与 visual-taste-lab 域重叠，待比对）、`vercel web-design-guidelines` 100+ 条规则集（与 ui-direction 互补，待逐项评估）、`awesome-dsh-plugin`/`dsh-market`/`dshfind`（DSH 生态目录，元列表，待扫） | 低成本后续轮 |
| P2 parked | 其余 ~390 URL（agent 框架主流项已在库；游戏/安全/jailbreak/论文列表与当前缺口无关） | 不读 |

## 2. Source Map

| # | Source | Type / License | Used For |
|---|--------|----------------|----------|
| R23-S1 | oil-oil/oil-ui v0.5.2（本地目录） | Skill / **MIT** | UI 方向方法论 + 对比页生成器 → 已落地 |
| R23-S2 | vitali87/code-graph-rag（5.2k★） | Engine / **MIT** | Tree-sitter + 编译器前端 + 运行时轨迹 overlay；dead-code-by-walking；ast-grep 改写 |
| R23-S3 | CodeGraphContext/CodeGraphContext（4.2k★） | Engine / **MIT** | 装饰器排除的死代码判定；GCF 紧凑输出；预索引包；live watch |
| R23-S4 | vercel-labs/agent-skills（31.8k★） | Convention / **MIT** | Agent Skills 格式（SKILL.md+scripts/+references）与我方 skill 布局一致性验证 |
| R23-S5 | trailofbits/skills（7.3k★） | Method / **CC-BY-SA-4.0** | 只取思想，不抄代码/文本：rust-review、insecure-defaults、fp-check、second-opinion |

## 3. Pattern → NeoTrix Mapping

| Source | Pattern（机制） | NeoTrix 映射节点 | 判定 + 消费者（R-P79） |
|--------|----------------|------------------|------------------------|
| S1 | 方向卡/差异检验/对比页/独立评审/减法（8 概念 des-ui 零命中） | **新增** `skills/design/ui-direction/` | 已落地 commit `8810b0dc`；消费者=走 skill 做方向探索的 Agent + 生成器直接可运行（18/18 测试 + CLI 烟测 52KB 产物） |
| S1 | `recommend_once.py`（pro 版付费推广） | — | **明确不吸**：吸入=植入第三方营销；已在 SKILL.md 记录，防后人误补 |
| S2 | 运行时轨迹 overlay（eBPF/测试 trace 合并动态分发） | `nt_calledges.py` 边表 | **Deferred**：静态边已落地 670K；动态 overlay 需重型 infra，记路线不做 |
| S2 | dead-code-by-walking-call-edges | 同上 | **Deferred**：需可见性数据（pub/test/derive 排除），当前边表无该字段；硬做=噪声门（已证否过一次） |
| S3 | 装饰器排除（`exclude_decorated_with`） | 同上死代码判定 | **Deferred**：同 S2 理由；但排除思想已记录，待可见性字段落地后启用 |
| S3 | GCF 紧凑输出（图查询结果省 token 62%） | `nt_calledges.py --compact` | **已落地**：TSV 单行输出，同查询同结果已验证（drain_outbox_once 例） |
| S3 | 预索引包（.cgc bundles） | `.project-map/edges-all.jsonl` | **明确不做**：176MB gitignored 已是上限，打包分发无意义 |
| S3 | live watch / 增量索引 | 边表再生流程 | **Deferred**：全量 ~30min 低频再生可接受；增量需按 crate 脏检查，路线已记 |
| S4 | Agent Skills 三件套布局 | `skills/*` 现有布局 | **Reinforce**：我方 `SKILL.md+references/+scripts/+tests` 与其一致，无需改 |
| S4 | web-design-guidelines（100+ 条代码评审规则） | `skills/design/ui-direction/` 互补位 | **P1**：它是代码层规则，我方是方向层方法，互补不重叠；逐项评估待后续轮 |
| S5 | rust-review（safe/unsafe 边界/panic-DoS/FFI + SARIF）| `neotrix-core`（Rust 主仓） | **P1**：思想可用（审查清单维度）；代码不可抄（CC-BY-SA）。待排期 |
| S5 | insecure-defaults（fail-open 并行审计 + 反驳验证器）| 静默失败门思想 | **Reinforce**：与 `check-silent-failure.sh` 同构（fail-open=静默失败类），互相印证 |
| S5 | fp-check（误报系统验证 + 强制门评审）| 全部门纪律 | **Reinforce**：与本仓「证伪测试」「注入探针」纪律一致 |
| S5 | second-opinion（独立复核）| oil-ui 独立评审 | **Reinforce**：两源收敛到同一纪律，可信度+1 |

## 4. Landings (this round)

| ID | Artifact | Verifies As |
|----|----------|-------------|
| L-R23-1 | `skills/design/ui-direction/`（8 文件，`8810b0dc`） | 上游 18/18 + 引入位置 18/18 + CLI 烟测 + layout/doc-drift rc=0 |
| L-R23-2 | `nt_calledges.py --compact` | 同查询 TSV/人类可读输出一致（drain_outbox_once 例） |

Deferred (recorded with reason): S2 运行时 overlay（重型 infra）· S2/S3 死代码判定（需可见性字段，硬做=噪声门）· S3 live watch（低频全量可接受）· S4 web-design-guidelines（逐项评估待后续）· S5 代码级吸收（CC-BY-SA 许可墙，只取思想）。

---

*End of Absorption Round 23*
