# NeoTrix 文档标准地图

> 版本: 1.0.0 | 生效: 2026-09-20
> **所有文档操作必须遵循本标准，禁止私自构建文件**

---

## 一、文档分类与存放位置

### 1. 项目根目录 (`/`)

| 文件 | 用途 | 维护者 |
|------|------|--------|
| `README.md` | 项目入口，500 字以内概述 | Owner |
| `CHANGELOG.md` | 版本变更记录 (Keep a Changelog) | Owner |
| `CONTRIBUTING.md` | 贡献指南 | Owner |
| `LICENSE` | 许可证 | Owner |
| `TODO.md` | **唯一任务清单** (结构化) | 所有会话 |
| `TODO.yml` | 任务机器可读格式。**由 `neotrix todo sync` 生成，禁止手改** —— `git_hook.rs:58-61` 提交时自动重新生成；被两个 pre-commit 钩子消费（`git_hook.rs:25-49`、`safety_tools/git-hook.sh:9`），**不可删** |
| `Makefile` | 构建命令 | Owner |
| `README.md` | 项目入口，500 字以内概述 | Owner |
| `RUST-STANDARDS.md` | Rust 编码标准正典 | Owner |
| `AGENTS.md` | Agent 守则（指针守恒） | 所有会话 |
| `results.tsv` | **进化实验账本** —— 由 `nt_evolution_exp.rs` 写入、`scripts/check-evolution-ledger.sh` 消费。⛔ 不是临时文件，勿删 | 进化系统 |
| `ARCHITECTURE-MAP-ROADMAP-V2.md` | 模块台账（更新规则为 **R-P199**，口径仅限 `neotrix-core` 的 L1–L6；`neotrix-neobot` 不占 L 层故不进此台账）；**§1-§7 数字永久陈旧，只取 §11 起** | Owner |
| `docs/architecture/NEOTRIX-MASTER-BLUEPRINT.md` | **唯一图纸入口**（D-00~D-15） | Owner |
| `docs/architecture/DIR-AUDIT-2026-09-27.md` | 目录架构审计（16 包依赖图 + 8 类重复类型）。**§六需加限定**：`nt_jev` 是活路径，勿当死代码 | Owner |
| `docs/architecture/DIR-REMEDY-2026-09-28.md` | **目录架构解法** —— 第二棵树 `neotrix/` 层归属显式化 + 唯一裁决表 | Owner |
| `docs/architecture/ABSORPTION-EXTERNAL-2026-09-27.md` | 外部吸收（trendshift 1041 仓 + 22 指定源） | Owner |
| `docs/architecture/EVOLUTION-ROADMAP-CODE-NODES-2026-09-28.md` | **正典**：进化路线的 file:line 支脉定位（109 仓 + trendshift 385 仓 + 5 arXiv；含 §执行状态 实测复核） | Owner |
| `docs/architecture/_superseded/EVOLUTION-ROADMAP-CODE-NODES-2026-09-27.md` | ⛔ 已归档，被 09-28 版取代（0.1/0.2/0.3 原始论证仍有效） | — |
| `docs/architecture/OWNERSHIP.md` | **唯一裁决表** —— 每条以**构造点**取证（非 `mod` 声明）；含 4 处「同名不同型」判定 | Owner |
| `docs/architecture/BATCH-FIX-CHECKLIST-2026-09-28.md` | **批量修复任务清单** A/B/C/D/E 五组 + 三道闸 + 回滚规程 | Owner |
| `docs/architecture/LESSONS-2026-09-28-measurement-and-dedup.md` | 经验沉淀：测量台纪律 / 机器读 ledger / worktree index / 同名非重复 | Owner |
| `docs/architecture/absorption-sources/` | **吸收源清单** —— 483 仓 CSV + 436 条榜单排名 + 许可台账 + 5 论文；复核任何结论从这里开始 | Owner |
| `docs/architecture/ABSORPTION-DSH-SIDEBAR-IM.md` | NeoBot 吸收正典记录（+12 模块 / IPC 97）；**neobot 改动的台账落点** | Owner |
| `docs/architecture/LESSONS-2026-09-27-scanner-trust.md` | 经验沉淀：扫描器告警 / 门记录腐化 / 并发写入 | Owner |
| `docs/architecture/LESSONS-20260929-checked-is-not-verified.md` | 经验沉淀：「我推演过」≠「我验证过」/ 能力断言看签名必翻车 / 提方案前先查自有 / 删入口≠删覆盖 / fixture 自造 / 授权≠免记账 | Owner |
| `sessions/HANDOFF-TEMPLATE.md` | 交接模板（**已入库**，`AGENTS.md` 引用） | 所有会话 |
| ~~`FUSION-ARCHITECTURE.md`~~ | ✅ **已于 2026-09-29 删除**（未跟踪文件，其"下一步"含已被证伪的"解决预存编译错误"） | — |
| `DOCUMENTATION-MAP.md` | 本文档 | Owner |

**禁止**: 根目录放置临时文件、会话笔记、分析报告

> ✅ **2026-09-29**：`apps/`（43 文件）与 `src-tauri/`（15 文件）两个残留
> 目录已移出到 `Neo/neotrix-archive/desktop-residual-20260929/`（`mv` 非 `rm`，
> 58/58 SHA-256 校验一致）。⛔ 其中 `src-tauri/` 那 15 个文件 **git 历史从无**，
> 含 2,496 行已被 `d5413335` 删除的冒烟测试 —— 恢复说明见该目录 `README.md`。
>
> ⛔ **本节不是散文了** —— `scripts/check-layout.sh` 把「根目录白名单 +
> `neotrix-core/docs/` 只许 `YYYY-MM-DD_` 前缀 + 与本文件的清单交叉校验」
> 变成可执行门，已接 pre-commit（拦新增，既有债记账）。
> **要在根目录放新东西前先跑它**：`bash scripts/check-layout.sh`。
> 有理由就扩脚本里的 `ALLOW_FILES` 并写明消费者 —— 别用完就留。

### 2. `docs/` — 文档主目录

```
docs/
（dev-rules.md 已删 → 正典 docs/standards/NEOTRIX-STD-1.0.md）
├── api/                      # API 文档 (mdbook)
│   ├── SUMMARY.md
│   ├── README.md
│   └── {module}.md
├── architecture/             # 架构文档
│   ├── ARCHITECTURE.md       # 目录结构与分类标准
│   ├── DATAFLOW.md           # 数据流
│   └── NEOTRIX-FULL-ARCHITECTURE.md
├── plans/                    # 设计方案 (日期前缀)
│   ├── YYYY-MM-DD-{topic}.md
│   └── 2026-09-22-wsd-wiki-index.md  # 外部挂载：WSD 企业 Wiki 索引（实体 /Users/neo/Downloads/wsd/wiki/docs/，19 篇，不复制内容进仓）
└── 2-PLANS/                  # 路线图
    └── ROADMAP-*.md
```

**规则**:
- `plans/` 文件必须用日期前缀: `2026-09-20-{topic}.md`
- `architecture/` 只放稳定架构文档，不放临时分析
- `api/` 文档从代码自动生成，不手写

### 3. `neotrix-core/` — 核心 crate

```
neotrix-core/
├── README.md                 # crate 概述
├── Cargo.toml
├── tests/                    # 集成测试
│   ├── nt_memory_integration.rs
│   ├── nt_shield_integration.rs
│   └── ...
└── benches/                  # 基准测试
    ├── memory_bench.rs
    └── ...
```

**禁止**: `neotrix-core/docs/` 存放研究笔记、分析报告

### 4. `skills/` — Agent 技能系统

```
skills/
├── SKILL.md                  # 技能系统总览
├── SKILL-SPEC.md             # 技能规范
├── index.json                # 技能注册表
├── {domain}/                 # 按领域组织
│   ├── SKILL.md              # 领域技能定义
│   └── {sub-skill}/
│       └── SKILL.md
```

**规则**:
- 每个技能目录必须有 `SKILL.md`
- 技能文件不超过 200 行
- 不在 skills/ 放非技能文件

### 5. `.neotrix/` — 内部运行时数据

```
.neotrix/
├── knowledge.db              # 知识库 (SQLite)
├── pending-absorb.json       # 待吸收队列
├── capability_*.json         # 能力注册表
├── experience/               # 会话经验 (自动)
│   └── session-*.md
└── agents/                   # Agent 运行时
```

**禁止**: 手动编辑 `.neotrix/` 下的 JSON/DB 文件

### 6. `.opencode/agent/` — Agent 提示词

```
.opencode/agent/
├── build.md
├── debug.md
├── nt-core.md
└── review.md
```

**规则**: 每个 agent 一个 md 文件，不超过 100 行

---

## 二、文件命名规范

| 类型 | 格式 | 示例 |
|------|------|------|
| 设计方案 | `YYYY-MM-DD-{topic}.md` | `2026-09-20-memory-distillation.md` |
| 路线图 | `ROADMAP-{scope}.md` | `ROADMAP-ARCHITECTURE-FUSION.md` |
| 分析报告 | `{topic}-analysis.md` | `gap-analysis.md` |
| 任务清单 | `TODO.md`（**唯一权威**，末节为统一进化清单） | 根目录唯一 |
| 交接文档 | `sessions/handoff-*.md`（**已入库**，`.gitignore` 白名单） | 每轮一份 |
| API 文档 | `{module}.md` | `memory.md`, `security.md` |
| 技能定义 | `SKILL.md` | 每个技能目录 |
| 变更日志 | `CHANGELOG.md` | 根目录 + 子 crate |

---

## 三、禁止行为

| # | 禁止 | 原因 |
|---|------|------|
| 1 | 根目录创建临时 md 文件 | 污染项目结构 |
| 2 | `neotrix-core/docs/` 放研究笔记 | 应放 `docs/plans/` 或删除 |
| 3 | 每个会话创建独立 TODO 文件 | 必须使用根目录 `TODO.md` |
| 4 | `skills/` 放非技能文件 | 破坏技能系统结构 |
| 5 | `.neotrix/` 手动创建文件 | 运行时自动管理 |
| 6 | 创建无日期前缀的方案文件 | 无法追踪时间线 |
| 7 | 创建超过 500 行的单个 md | 应拆分为多个文件 |

---

## 四、文档生命周期

```
创建 → 审核 → 合并 → 归档/删除

1. 创建: 按本标准选择正确位置
2. 审核: 检查命名、长度、格式
3. 合并: PR 合并到主分支
4. 归档: 过期文档移至 git 历史，不保留
```

---

## 五、执行检查清单

新会话开始时检查:

- [ ] 读取 `TODO.md` 了解当前任务
- [ ] 读取 `docs/standards/NEOTRIX-STD-1.0.md` 了解开发规则（正典；旧 `dev-rules.md` 已于 2026-09-29 删除）
- [ ] 读取 `DOCUMENTATION-MAP.md` 了解文档标准
- [ ] 不在禁止位置创建文件
- [ ] 文件命名符合第二节规范
- [ ] 完成任务后更新 `TODO.md` 状态

---

## Diátaxis 象限划分 (SIM-25, NTS-D07 配套)

> 指南针：动作×认知 / 习得×应用。只分类现有文件，不建空目录（Diátaxis 工作流警告）。

| 象限 | 回答 | 本仓归属举例 |
|------|------|-------------|
| Tutorial（习得＋动作） | 能带我走一遍吗？ | 按图施工手册开头的四步法（BLUEPRINT） |
| How-to（应用＋动作） | 怎么做 X？ | scripts/ 用法、Makefile targets、门禁接线表 |
| Reference（应用＋认知） | X 是什么？ | NT-STD 条款、模块 API 文档、ADR 索引、遥测字段表 |
| Explanation（习得＋认知） | 为什么这样？ | ABSORPTION 系列、SIM 记录、FUSION/DESIGN 分析文档 |

新文档落笔前先判象限，一篇只属一象限；跨象限内容拆分，不混合。
