# 吸收：Anthropic《The AI-native SDLC playbook》(2026-08-21)

> **来源**：`https://claude.com/blog/the-ai-native-sdlc-playbook`
> 作者 Louis Claxton，Anthropic Applied AI 团队，2026-08-21，阅读时长 5 分钟。
> **吸收方式**：读原文 → 与本仓实测对账 → 只接**缺口** → 同会话接到生产可用（R-P79）。
> **性质**：吸收**思想**（流程论点），不复制代码（原文无代码，是 playbook 文档）。

---

## 1. 一句话核心论点（也是全文唯一的真论点）

> **代码不再是瓶颈，人速的环节才是。**
> build 阶段塌缩到小时级，于是瓶颈移到它左右两侧：**plan / review+test / deploy**。

原文给出的三个后果（本仓据此定优先级）：

1. 瓶颈移到 build 左右的**人速步骤**；
2. **控制措施与现实脱钩**——逐行人审在 agent 写大部分 diff 之后跟不上；
3. **治理成本上升**——例外仍走每周/每月开会的委员会。

⇒ 解法不是加人，是把每阶段产物**提交进版本控制**，并让**每次提交成为审计轨迹**：
> "The chain of commits is also the audit trail: who asked for what, what the
> agent produced, and who approved it."

---

## 2. 六阶段对照表（原文 §「The shifts across the six stages」）

| 阶段 | 传统 | AI-native | 本仓 2026-10-06 实测 |
|---|---|---|---|
| Plan | 委员会收集、手写 | 合成痛点写入 `intent.md`，人可读机可动 | **缺口** → 已接（工件链 §2） |
| Design | 分析师写、设计师解 | 一次会话出 spec，受 skills 约束 | 部分有（`skills/`），无工件 |
| Build | 手写测试与代码，文档事后补 | AI 生成；知识存 `CLAUDE.md` + skills | **已有且更强**（见 §4） |
| Test | 阶段边界的 QA 门 | 持续 evals 织进实现 | **缺口**（evals 无 runner、未进 CI） |
| Deploy | 人审每行，治理在 review 周期 | 分层 agent 审查 + hooks 当审批门 | **缺口**（无 REVIEW.md） |
| Maintain | 人盯生产 | agent 监控，越界写回 `intent.md` | **有 autonomy 分级**，band 检测缺 |

---

## 3. 本仓真缺口（**实测数字，非推理**）—— 本次吸收的靶子

| # | 缺口 | 实测证据 |
|---|---|---|
| **G1** | **无每变更的提交工件** | `intent.md` 全仓命中 **0**；`proto-spec` 命中 **0**；`docs/plans/` 66 件全是**主题级**；`sessions/handoff-*.md` 70 件全**回顾性**（`HANDOFF-TEMPLATE.md` §3/§4/§8） |
| **G2** | **agent 配置无回归门** | `ci.yml:3-11` 的 `on:` **无 paths 过滤** ⇒ 改 `skills/index.json` 的 PR 会跑 5 分钟 cargo 却零配置校验；`nt_find.py --audit` 只在 `pre-commit:97`（`--no-verify` 可跳过）；`check-skill-gate.sh` 在 CI 与 Makefile **双双零引用** |
| **G3** | **无版本化审查策略 + 两套严重度并存** | 根无 `REVIEW.md`；`.opencode/agent/review.md:16` 自造 3 级 `Blocker/Warning/Info`，而代码库正典是 9 级 `Severity`（`crates/neotrix-types/src/core/shared_types.rs:8`，`numeric()` 0–8）—— **无映射表** |
| **G4** | **过程指标全空** | 10 个 `*-baseline.txt` 全部只数**代码缺陷**；`grep DORA` 命中全是方案文档非仪表；`REVIEW.md §6` 三项指标原文即标注「未接线」 |

---

## 4. 本仓**已经更强**的部分（**不重复造** —— 原文没有的）

原文的 playbook 里**没有**下面这些。这是本仓的独立贡献，吸收时**不得覆盖**：

| 本仓机制 | 原文对应 | 为什么本仓更强 |
|---|---|---|
| `scripts/gate-registry.tsv` + `check-gate-satisfiable.sh` | 无 | 原文只有「hook 当确定性层」，**没有「门是否可信」的概念**。本仓强制两道判据：① 非空（注入违规必红）② 可满足（合规输入必绿）。原文的 skill/hook 两分法解决不了「门是假门」 |
| `check-gate-satisfiable` 的 `injectable`/`opaque` 诚实登记 | 无 | 「我测不了」是合法答案，「我没测」不是 |
| 12 道门的 anti-vacuity 自检 + 9 份 `scripts/probes/*.sh` | 原文只有 1 个 evals 例子 | 本仓把「门能不能变红」变成**可执行断言**（`assert_gate_red`） |
| `nt_gate_coverage.py`（门必须接 CI 或带理由豁免） | 无 | 直接对应原文的「configuration deserves regression testing」的**接线**面 |
| `AutonomyLevel`（ReadOnly/Constrained/Supervised/Autonomous）+ `required_autonomy()` 13 个 `TaskType` 映射 | 原文 Stage 6 只给 `bands.yaml` 的 YAML 形态 | 本仓已把它做成 Rust 类型 + 已接线（`entry/headless.rs`、`entry/todo.rs`、`entry/wiki.rs`） |

⛔ **最后一行是本轮最容易踩的坑**：原文 Stage 6 看上去给了「分层自治」这个好东西，
但**本仓早就有而且已经是生产接线**。照抄 `bands.yaml` 会造出第二个自治机制。
（R-P79 的反面：**导出 ≠ 接入**；同理，**已有 ≠ 需要照抄**。）

---

## 5. 实际接了什么（产物清单 + 接线证据）

| 产物 | 补的缺口 | 接线证据 |
|---|---|---|
| `docs/plans/_TEMPLATE.md` | G1 | 被 `check-agent-config.sh` C1/C4 消费 |
| `REVIEW.md` | G3 | 被 `.opencode/agent/review.md:12,17,24,27` 消费；`check-layout.sh` ALLOW_FILES 已登记 |
| `scripts/check-agent-config.sh` | G2+G4 | `gate-registry.tsv` 第 20 行（injectable）；`.github/workflows/ci.yml` 有跑点；`.neotrix/task-index.json` 第 79 条可被 `nt_find` 检索 |
| `scripts/probes/check-agent-config.sh` | 非空性 | 2 处注入实测见 §6 |
| `scripts/artifact-chain-baseline.txt` | 棘轮基线 | 66 件存量，**不追溯** |

### 关键设计决策（以及为什么）

**决策 1：一件工件三段状态，不造三个文件。**
原文是 `intent.md` → `spec.md` → `plan.md` 三件。**照抄会造第二套真源**，
违反指针守恒（`AGENTS.md` 开篇原则）。改为一件工件 + `status:` 字段推进，
状态即阶段标记 ⇒ 机器可判，链不散。

**决策 2：工件链是棘轮，不是普查。**
既有 66 件工件**一节都不全**。普查即**出生恒红** ⇒ 恒红的门等于没有门
（本仓门纪律，`.githooks/pre-commit:112` 原话）。故只拦增量。
⛔ 代价已写进门输出：**「棘轮管辖 0」不是「全仓合规」，是「无增量」**——
拿它汇报覆盖率是错的。

**决策 3：严重度对齐代码库正典，不引入新词表。**
原文的 Important/Nit 二分**没有映射规则**。本仓已有 `Severity::numeric()`，
且 `from_score` 本来就把 5.0 当分界（`shared_types.rs:24-26`）
⇒ 分桶直接用正典权重 5 划线，**不新造规则**。

**决策 4：不复用 `check-skill-gate.sh`。**
它 `--strict` 恒红（存量未清，`gate-registry.tsv:27` 自述）。接进 CI = 造恒红门。
本门只守**引用完整性**，不碰它的判据。

---

## 6. 非空门实证（**跑出来的，不是看出来的**）

`scripts/probes/check-agent-config.sh`，两处注入各自独立证明判别力：

| 注入 | 期望 | 实测 |
|---|---|---|
| 新增缺节工件（只写 3/9 节 + `status:` 行） | exit 1 + **指名该工件** + 点名缺失小节 | ✅ 三项全中 |
| 去掉 review agent 的 `REVIEW.md` 引用 | exit 1 + **指名断链消费者** | ✅ 两项全中 |

⚠️ 断言**必须指名注入对象**，不能只判 rc —— 否则门若因别的原因红，
探针会自证循环（`gate-registry.tsv` 里 `check-unwrap` / `check-silent-failure`
两条 note 记录的历史教训）。

---

## 7. 明确**不吸收**的部分（及理由）

| 原文机制 | 不吸收的理由 |
|---|---|
| `intent/` 独立仓库 | 原文自己都说：单产品用产品仓里的 `intent/` 目录即可。本仓已有 `docs/plans/`（66 件 git 跟踪），另起一套是第二真源 |
| `bands.yaml` 分级自治 | **本仓已有** `AutonomyLevel` 且已接线（§4）。照抄造第二机制 |
| managed settings / `permissions.deny` / sandbox | 这是**企业 MDM 分发**问题。本仓是单机自管仓，无 MDM 面 |
| Claude Tag（Slack on-call）、Claude Security（托管扫描） | 外部 SaaS，接入即引入外部依赖与费用；本仓无 Slack/托管扫描位 |
| 每次 play 的 leading/lagging 指标 | 原文给的是**指标口径**，不是实现。⇒ **只落地本仓真能 git 复算的 3 项**（`scripts/ops/nt_sdlc_metrics.py`，2026-10-06 同会话补齐），另 3 项如实标 `UNMEASURED` 并写明为何测不出（`REVIEW.md` §6.2）。**不造没人跑的空仪表**，也不把测不出的报成 0 |
| 20–50 条真实任务的 eval 套件 | 需真实 agent 非交互运行 + API key + 预算。本仓 `evals/gaia_mini` 已声明「模型实测：按需，不进 CI」（`evals/VERIFICATION.md:24`）——**同一道纪律**：不把「没跑」记成绿 |

---

## 8. 本轮顺手查出的**既有**缺陷

### 8.1 `check-ci-refs.sh --strict` 恒红 —— **已按规程补记，修掉了**

```
.github/workflows/ci.yml uses=actions/setup-python@v5
  (未登记于 .../nt_shield/provenance/external-inputs.json)
```

- **真阳性**，非误报：`actions/setup-python` 确实不在 13 条溯源清单里
  （`check-ci-refs.sh:198-199` 的匹配逻辑已读现场核实：先剥 `@version` 再比对 name）。
- 出处 `ci.yml:50`，`git show HEAD:.github/workflows/ci.yml` 已确认**早于本次改动**。

**处置（不是编哈希）**：该清单 `_comment` 自带**离线可复现流程**，
2026-10-06 有网 ⇒ 按流程**实测**而非编造：

| 字段 | 值 | 取得方式 |
|---|---|---|
| `commit` | `a26af69be951a213d495a4c3e4e4022e16d87065` | `git ls-remote --tags … refs/tags/v5` |
| `sha256` | `6318d936…25a09` | `curl -sL codeload.github.com/…/tar.gz/<commit> \| shasum -a 256` |
| `tarball_bytes` | `1569541` | 同一次下载的字节数（人工复核 sha256 是否算错对象） |

⛔ **两次独立下载 sha256 完全一致**（可复现性实测），且 `file` 确认是真 gzip tarball
（排除「拿到错误页还照样算哈希」这种最坏情况）。
⇒ `provenance_check.sh` 14/14 PASS；`check-ci-refs.sh --strict` **rc=0**。

⚠️ 清单 `_comment` 已警告：`@v5` 是**可移动 tag**，上表只对记录那一刻成立；
真要不可变须把 `uses:` 改成 `@<commit SHA>`（**属独立决策，本轮未做**）。

### 8.2 仍未修（需要 cargo 或产品码改动，非本轮范围）

| 门 | 状态 |
|---|---|
| `check-doc-drift.sh --strict` | NEW offender `neotrix-core/src/l0_substrate/nt_core_event_bus.rs`（既有/他窗） |
| `check-unwrap.sh --strict` | 存量债（`gate-registry.tsv` 已记「当前恒红，NEW 5 / STALE 6」） |
| `check-silent-failure.sh --strict` | 同上 |
| `check-commit-deletions` 探针 | `check-doc-claims` 探针 rc=2 ⇒ 门有效性未获证（既有） |

---

## 9. 原文里**没写**但值得记住的一句

> "A skill is a control, though an advisory one... A policy that must always
> hold needs something deterministic behind the skill, such as a hook that
> blocks the action. **The skill makes violations rare and the hook makes them
> close to impossible.**"

这句与本仓 `check-skill-gate.sh` 的处境**完全对上**：skill 索引存在（59 条）
却无门拦（CI/Makefile 双零引用）⇒ 违规既不「rare」也谈不上「impossible」。
本仓的解法方向与原文一致：**skill 之上必须有确定性层**。
但本仓走的是**登记制**（`gate-registry.tsv` + `nt_gate_coverage.py`）
而非逐技能写 hook——理由见 `gate-registry.tsv` 头部「为什么 probe 用 eval」：
规则一多就没人维护（`Agentero` 与本仓 dependency 门都只有 3 条规则，
**正是因为规则一多就没人维护**）。