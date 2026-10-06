# REVIEW.md — 审查策略（review policy）

> **吸收源（思想，非代码）**：Anthropic《The AI-native SDLC playbook》(2026-08-21)
> Stage 5 Deploy §「AI in the PR review loop」第 2 步（把审查策略写成仓库根下
> 版本化的 `REVIEW.md`，含 passes / Important 判据 / nit 上限 / 不报清单）。
> 完整原文事实源：`docs/architecture/ABSORPTION-AI-NATIVE-SDLC-2026-10-06.md`。
>
> **消费方**（改这份文件前先确认谁是活的）：
> - `.opencode/agent/review.md`（review 子代理，审查协议正文所在）
> - `scripts/check-agent-config.sh`（守「review agent 必须指向本文件」这条引用完整性）
>
> **指针守恒**：规矩在本文件；**门怎么跑在 `AGENTS.md`**，不在这里重复。

---

## 0. 本文件解决的一个真实矛盾（2026-10-06 实测）

| 事实 | 出处 |
|---|---|
| 代码库的严重度正典是 **9 级** `Severity` | `crates/neotrix-types/src/core/shared_types.rs:8` |
| 每级有 `numeric()` 权重 **0–8** | 同上 `:42-48` |
| 分数 ≥5.0 判 `Medium` 起 | 同上 `:24-26` |
| review 子代理却用**自造的 3 级** `Blocker/Warning/Info` | `.opencode/agent/review.md:16,31` |

⇒ **两套严重度词表并存**，且 agent 那套与代码库任何类型都对不上：机器消费的
`Severity` 九级，人/agent 消费的 `Blocker` 三级，**没有映射表**。
本文件把审查口径**对齐到代码库正典**，并按 `numeric()` 划 Important / Nit 两桶。

---

## 1. 严重度口径：对齐 `Severity::numeric()`

不新造词表。分桶只用正典已有的 `numeric()` 权重，边界取 `from_score` 的 5.0：

| 桶 | 级别 | numeric | 含义 |
|---|---|---|---|
| **Important** | `Critical` | 8 | 必须合并前处理 |
| | `High` | 7 | |
| | `Error` | 6 | |
| | `Medium` | 5 | 分界线下沿（`from_score` 的 5.0） |
| **Nit** | `Warning` | 4 | ≤5 条，超出只报计数 |
| | `Low` | 3 | |
| | `Info` / `Informational` | 2 / 1 | |
| | `Pass` | 0 | |

**「Important」在本仓的确切含义**：会破坏行为、泄漏数据、或违反一条已成文公理
（零 unsafe / 层归属 / 指针守恒 / 提交必声明删除意图 / 偏离计划必同 commit 改工件）。
**命名、格式、措辞一律是 Nit。**

⛔ 分界不是新发明：`from_score` 本来就把 <5.0 的丢给 Info/Informational。
本文件只是把这个既成事实写成两桶，省掉 agent 每次自己现编一档。

---

## 2. Passes（每次审查跑的三遍，各打标签）

沿用 review agent 已有维度，但**改成引用本仓真实门名**，不再泛泛而谈：

| Pass | 查什么 | 依据（真实出处） |
|---|---|---|
| **bugs** | 逻辑错误、边界破、回归。含「并发」专项：是否跨 worktree / 共享 index 假设 | `AGENTS.md` §2 并行公约 |
| **safety** | 生产码 `unwrap`/`expect`/`panic`、被丢弃的 `Result`、0 字节 `.rs`、孤儿目录 | `#![forbid(unsafe_code)]`；`check-unwrap.sh` / `check-silent-failure.sh` / `check-truth-surface.sh` |
| **commitment** | 改的是不是被要求的那件事：与工件的 `## 改动文件` 是否一致；删除是否有 `DELETION-INTENT`；偏离是否同 commit 改了工件 | `docs/plans/_TEMPLATE.md` §4；`.githooks/prepare-commit-msg` |

**第三遍是本仓最有价值的一遍，也是 playbook 说的「human attention 上移」**：
前两遍能机械化的都已被门挡住（见 §3），人真正要判的是
**「这次 diff 是不是实现了要实现的东西、风险能不能接受」** ——
而那个基线只存在于工件里。

---

## 3. 不报清单（Do not report）= 门清单

**已被确定性门拦住的项，审查不再复述。** 理由不是省事：

1. **重复记账**。`check-unwrap.sh` 已在 CI 阻断新 `unwrap`；审查再报一遍，
   同一处缺陷在 PR 里出现两次，读者不知该看哪个。
2. **会训练人忽略红色**。若审查噪音里混着门早该拦下的项，真 Important
   会被淹没（本仓门纪律的同一条，形式不同）。
3. **审查额度是零和的**。nit 上限见 §4；把额度花在复述门上，真问题就没额度了。

**已被门覆盖、审查不报**（CI 实跑项，2026-10-06 核对 `ci.yml`）：

| 门 | 管什么 |
|---|---|
| `check-unwrap.sh --strict` | 新增生产码 unwrap/expect/panic（712 处存量已棘轮化） |
| `check-silent-failure.sh --strict` | 丢弃 `Result` 且无观察通道 |
| `check-layer-deps.sh --strict` | L0–L6 跨层依赖方向 |
| `check-truth-surface.sh --strict` | 0 字节 `.rs` / 未声明 test module / 入库的构建产物 |
| `check-fresh-build.sh` | 干净克隆能装上、能加载 workspace |
| `check-test-baseline.sh --strict` | 测试失败台账（基线为空 ⇒ 任何新失败即红） |
| `check-ci-refs.sh --strict` | CI job 指向 git 未跟踪的路径 |
| `check-doc-claims.sh` | 文档断言「零消费者」而代码反证 |
| `check-gate-satisfiable.sh --strict` | 门本身可满足（非空 + 可满足） |
| `nt_gate_coverage.py` | 门已接 CI 或带理由登记豁免 |
| `nt_security_wiring.py --strict` | 资产未被安全接线（棘轮） |
| `check-commit-deletions.sh` / `prepare-commit-msg` | 删除未声明意图 |

⛔ **本清单要跟着门走**：新增门进 CI 后，把它加进这张表。
「不报清单」腐化 = 审查开始复述门 = 本文件失效。
`scripts/check-agent-config.sh` 守的是**本文件被引用**，
**这张表与 CI 的同步由人工 review 时确认** —— 一道纯文本清单无法自我验证（下面讲原因）。

---

## 4. 上限与职责分离

**nit 上限：每次审查最多 5 条 Nit，超出只报计数。**
（Nit 定义见 §1：`numeric() < 5`。）

**职责分离（playbook Stage 5 的核心治理声明）**：
> **写出代码的 agent 没有任何路径可以批准它自己的代码。**

本仓形态：审查 findings 不自行批准任何东西；合并批准只由人经 pre-commit /
pre-push hook 完成。agent 只能把门跑绿，**门不做批准判断**。

⛔ 因此 review agent **不得**输出「approve / 通过 / 可以合并」——
它没有这个权限，写出来就是伪造治理。审查代理只报发现，不发批准。

---

## 5. 每条发现的格式

与 review agent 现有输出一致（`.opencode/agent/review.md:31-35`），
补齐 severity 取值与 pass 标签：

```
### [Severity::<级别>] <pass> · <标题>
- **File**: path/to/file.rs:42
- **Issue**: <现象>
- **Fix**: <具体到能直接改>
- **依据**: <公理 / 门 / 工件节名>
```

`依据` 是必填。**说不出依据的发现不要报**——那不是发现，是猜测。

---

## 6. 测量（playbook 要求每个 play 有先行 + 滞后指标）

### 6.1 测得出的（本仓实测，git 可复算）

跑 `python3 scripts/ops/nt_sdlc_metrics.py --head 200`。它是**报告器不是门**。

| 指标 | 口径 | 2026-10-06 实测 |
|---|---|---|
| 先行 | **工件覆盖率** —— product-code commit 前后 ≤3 个 commit 内是否有新建工件 | **0.9%**（1/111） |
| 先行 | **工件→diff 漂移** —— 下一个 commit 是否越过了工件的「改动文件」清单 | **NO_BASELINE**（11 件存量工件无该节） |
| 滞后 | **返工代理** —— 同文件被 ≥3 commit 触碰 | **1.5%**（5/329） |

⛔ **工件覆盖率 0.9% 不等于「机制失败」**：它衡量**增量是否守规矩**，
不衡量历史欠账。机制 2026-10-06 才接上，历史 2622 个 commit 一件都不补。
⛔ **返工是代理指标**：分不清「迭代设计」与「真返工」，**不是 DORA**。

⚠️ **改它的度量逻辑前先跑 `--self-test`**：该自检在合成夹具上断言
「coverage>0」与「drift 触发并指名未列文件」。没它就无法区分「真的 0」与「写坏了恒 0」。
⇒ **这个自检当场抓到我自己的两个真 bug**：① 窗口建在只含 code commit 的列表上
⇒ 工件若单独成一个 commit 就永远进不了窗口；② `git log -n N` 配 pathspec
数的是「匹配的 N 个」而非「全部的前 N 个」⇒ 两个窗口不一致，KeyError。
**两个都是跑出来的，不是读代码看出来的。**

### 6.2 测不出的（如实标注 `UNMEASURED`，不报 0）

| 指标 | 为何测不出 |
|---|---|
| PR 首次审查耗时 | 本仓 agent 走本地工作树 + `git commit --only`，**不走 GitHub PR 流程** ⇒ 无 first-review 事件 |
| 审查发现被门重复报出的比例 | 需要结构化审查发现台账；发现目前只存在于对话里，**无落盘格式** ⇒ 无数据源 |
| 合并前拦下 vs 逃逸到生产的缺陷 | 需要事故台账；本仓**无生产环境** ⇒ 分子分母都没有 |

**诚实声明**：这三项**至今未接线**。把它们写成 0 就是在撒谎
（`evals/VERIFICATION.md:8-9`：没跑 ≠ 通过）。