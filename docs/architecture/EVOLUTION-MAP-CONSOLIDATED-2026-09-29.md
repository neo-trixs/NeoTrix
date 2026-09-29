# NeoTrix 完整进化地图 — 单一真源（2026-09-29 合并定稿）

> **本文件是三份路线的合并定稿**：
> `EVOLUTION-ROADMAP-CODE-NODES-2026-09-28.md`（阶段 0–5）·
> `EVOLUTION-ROADMAP-CODE-NODES-2026-09-29.md`（N-1…N-13）·
> `BATCH-FIX-2026-09-29.md`（任务清单）。
> 前两份**保留作各轮取证留档**，但**排期与状态以本文件为准**。
>
> **口径纪律（三轮不变）**：
> 1. 每条状态都是**实测**，不是文档转述。凡本文写的行号/计数，**接手前请重跑文末的复核命令**。
> 2. 「自进化机制不是护城河，**能证明自进化是否有效**才是」——
>    已从 11 仓扩到 **30 仓**验证（`ABSORPTION-AGENT-ARCH2` §1.1）。
> 3. **没有验收的条目不叫路线，叫愿望。** 每条都有可执行验收。
> 4. ⛔ **恒红的门比没有门更坏** —— 它训练人忽略红色。

---

## 0. 一页纸：现在在哪、下一步做什么

| | |
|---|---|
| **已交付（本轮）** | 拆掉 **5 个恒红幻影门**；`check-truth-surface` 扩域（**212 个 `.rs` 从不被编译**首次可见）；新建 `check-ci-refs.sh` |
| **最大未做** | **N-5/N-11/N-12 测量面**（唯一能让 NeoTrix 领先而非跟随的）<br>**N-3 空账本**（`check-test-baseline.sh` baseline = **0 字节**，CI 正在调它） |
| **最危险** | **N-2 的 221 条未编译文件**：多数是已归档旧引擎，但 **(c) 类加回去会让干净检出编不过** |
| **最被低估** | **N-4**：`SkillInvocationPolicy` 被第一轮记成「核心代码」，实测**零消费者**，是**门** |
| **下一步（批次 A，零依赖）** | N-3 → N-5 → N-4。三件都不碰 `.rs`，当前树即可做 |

---

## 1. 阶段总览与依赖

```
┌─ 批次 A：让「绿」有意义（零 .rs，当前树可做，~2 天）────────────┐
│  A1 N-1 CI 幻影门        ✅ 本轮完成                              │
│  A2 N-2 未编译代码可见    ✅ 本轮完成（存量入账，处置待裁决）        │
│  A3 N-3 test 账本填充     ⬜  ← 现在做                             │
│  A4 N-5 claims 数字追溯   ⬜                                      │
│  A5 N-4 skill policy 接线  ⬜                                      │
└────────────────────────────┬───────────────────────────────────┘
                             │ 依赖：要在已可信的门上加断言
┌─ 批次 B：测量面（最高长期价值，~1 周）───────────────────────────┐
│  B1 N-12 门可满足性元门    ⬜  ← 优先，它是元门                     │
│  B2 N-11 门记录带 env 指纹  ⬜                                      │
│  B3 0.2 证伪门（预注册/四事实/正负都提交）⬜                        │
│  B4 0.3 maturity 降级落盘  ⬜ 资产已在，纯接线                       │
└────────────────────────────┬───────────────────────────────────┘
                             │
┌─ 批次 C：策略面（⚠️ 有并发冲突，需先协调）───────────────────────┐
│  C1 N-6 三态工具策略       ⬜                                      │
│  C2 N-7 StopReason 正交    ⬜                                      │
│  C3 1.4 非不可宽化策略地板   ⬜                                      │
│  C4 1.1 DNS qtype 白名单    ⬜                                      │
│  C5 1.2 attempt/outcome 解耦 ⬜                                    │
│  C6 5.2 MCP capability 协商  ⬜                                      │
└────────────────────────────────────────────────────────────────┘

┌─ 批次 D：记忆与检索（依赖 A 完成）──────────────────────────────┐
│  D1 2.1 experience_tree supersession  ⬜                          │
│  D2 2.2 nodes 表对齐 temporal_facts  ⬜ ⛔ 风险最高                │
│  D3 N-10 delta-op + Provenance 默认    ⬜                          │
│  D4 N-9 多因子打分缺失塌 1.0          ⬜                          │
│  D5 N-8 缓存键含代码                  ⬜ 真值风险最低，最后做       │
└────────────────────────────────────────────────────────────────┘

┌─ 批次 E：2026-09-28 遗留（部分已被证伪，见 §4）──────────────────┐
│  E1 0.1 ToolRegistry 3→? / SkillRegistry 4→1   ⬜                 │
│  E2 阶段 2 桌面执行回路（2.1–2.5）             ⬜ src-tauri 已归档  │
│  E3 阶段 3 成本与上下文（3.1–3.3）             ⬜                  │
│  E4 5.1 GUI 元素寻址（NoopBackend 仍是唯一后端）⬜                  │
└────────────────────────────────────────────────────────────────┘
```

**强依赖只有 5 条**，其余可并行：
`A(绿可信) → B(测量) → 阶段4 自进化` ·
`D1 → D2` · `C(策略) 独立` · `E 独立`

---

## 2. 批次 A —— 让「绿」有意义（零 `.rs`，当前树可做）

### ✅ A1 · N-1 CI 幻影门（**本轮完成**）

| | |
|---|---|
| **做了什么** | 删 `ci.yml` 的 `frontend-tests`/`frontend-coverage`/`frontend-build`/`e2e`（指向 `.gitignore:136` 整目录忽略的 `neocodex-frontend/`，0 跟踪）+ 整个 `docs-deploy.yml`（`docs/package.json` 已于 `477bf669` 删除）；新建 `scripts/check-ci-refs.sh` 并接入 `ci.yml` |
| **验收** | 干净树 `--strict`=0；注入幻影 job 后 `--strict`=1 且指名行号。**双向验过** |
| **复核命令** | `bash scripts/check-ci-refs.sh --strict; echo $?` |

### ✅ A2 · N-2 未编译代码可见（**本轮完成**，处置待裁决）

| | |
|---|---|
| **做了什么** | `check-truth-surface.sh` class 2 从「只限字面叫 `tests` 的目录」扩到**全部含 `mod.rs` 的目录**（381 个）+ 新增 `UNREACHABLE` 类；265 条入 baseline 成棘轮 |
| **方法（关键）** | 从每个 crate root 出发的**传递 mod 可达性**（含 `mod X;`/`mod X {}`/`#[path]`/`include!`）<br>⛔ **不用 dep-info**：它只覆盖单个 target（列 2,225，磁盘 2,550），会大面积误报 |
| **实测** | `UNDECLARED 53` + `UNREACHABLE 212`；**221 条全部是 git 已跟踪文件**（非任何窗口 WIP） |
| **⛔ 处置纪律** | 「未被编译」≠「功能缺失」。(a) 已归档旧引擎→删；(b) 忘了加 mod→**先问能否编译**；(c) **声明被注释掉**→`nt_memory/mod.rs:54` 注「内部编译错误待修复」，**加回去会让干净检出编不过** |
| **复核命令** | `bash scripts/check-truth-surface.sh 2>&1 \| sed -n '2,3p'` |

### ⬜ A3 · N-3 test 账本填充（**下一步**）

| | |
|---|---|
| **支脉节点** | `scripts/check-test-baseline.sh` + `scripts/test-failures-baseline.txt`（**实测 0 字节**）· 消费方 `ci.yml` `test` job |
| **为什么是 P0** | `handoff-20260928-consolidated §3 T1` 写「账本已无 flaky，技术上可拦」—— 那**以账本有内容为前提**。空账本 + `--strict` = **零容忍**，而 `consolidated §1` 自己记录「57 条失败（37 条在主工作树已修好、只是从未入库）」⇒ **现在开 `--strict` 会立刻红** |
| **行业教训** | `cline`（69.5k★）：3 层 eval 框架（`pass@k` **和** `pass^k` / `FLAKY` 一等态 / 失败分类器带 issue 链接）→ CI 被 `removed`、smoke `disabled`、`benchmarks/tool-precision/DEPRECATED.md`。**测量基础设施被一次重构孤儿化，没有任何东西会告诉你** |
| **动作** | ① 从**干净检出**实测填充（**不是主工作树** —— `LESSONS-20260928-fresh-checkout`）② 非空门证明 |
| **验收** | 干净检出 `--strict`=0；注入一条失败测试后 `--strict`=1 且指名该测试 |
| **回滚** | `git checkout -- scripts/test-failures-baseline.txt` |

### ⬜ A4 · N-5 claims 数字追溯（`kev/scripts/verify_claims.py`）

| | |
|---|---|
| **支脉节点（地基已在）** | `scripts/ops/nt_manifest.py:58-79` `env_fingerprint()` · `:96-111` `cmd_add` · `:141-158` `cmd_stale`(exit 1) · `:161-194` `cmd_audit` · 已挂 `.githooks/pre-commit` 阻塞态 |
| **缺口** | 它验证**引用的位置**（`file:line` 存在且行号 ≤ 文件长度），**不验证引用的数字** |
| **本轮抓到的实证（4 处）** | ① `ABSORPTION-AGENT-ARCH-2026-09-28.md:13` 「frontmatter 41/58」→ **32/59**<br>② `check-skill-gate.sh` 头「67 个 SKILL.md」→ **68**；「24/67 有 frontmatter」→ **34/68**<br>③ 同文件「58 条存量」→ **59**（`categories.*.skills` 之和）<br>④ `TODO.md:526` 引 `registry.rs:459` → 真实 **`registry.rs:466`**，且**已接 CI 阻塞态**（不是 ⬜）<br>（①②③本轮已在 `check-skill-gate.sh` 注释里改正并记下复算命令） |
| **动作** | claim 加 `printed`（文档里的字面量）+ `source`（原始结果文件）+ `select`（键过滤）+ `scale`；`audit` 加「印出的数字能否按**印刷精度**从 source 复算」<br>派生量照抄 kev：`macro_mean`、`over_requested`（折算回全量，**被拒的算错**） |
| **验收** | 把上面 4 个错数字写进 claims 后 `python3 scripts/ops/nt_manifest.py audit` 必须 exit 1 并指名 |

### ⬜ A5 · N-4 skill policy 接线（第一轮误标）

| | |
|---|---|
| **支脉节点** | `neotrix-core/src/skill_loader.rs:107-142` `SkillInvocationPolicy` · `:197-201` `is_official_converged()` · `skill_registry.rs:105-170` `route_entity_aware()` · `crates/neotrix-gateway/src/skill_registry.rs:157` |
| **实测（证伪第一轮）** | 三个 `visible_to_model()`/`visible_to_user()`/`is_trusted_only()` **零消费者** ⇒ 第一轮记的「**核心代码** 4 测试」**不成立**，是**门**（那 4 个测试是自证）<br>`is_official_converged()` **零调用者** = 死代码<br>`route_entity_aware()` **生产零调用**（只有 2 个测试位）<br>gateway 那个 626 行「真典」，唯一调用方 `nt_crystal_serve.rs:1303`，其 `with_defaults()` 指向 `$CWD/.neotrix/skills` + `~/.neotrix/skills` —— **两处都不含 `index.json`** ⇒ **它扫不到东西** |
| **动作** | ① 把 policy 接到 `SkillLoader::list_skills()` 返回上 ② 删 `is_official_converged()` ③ 裁决另两个：**要么接、要么删** |
| **验收** | `grep -rn "visible_to_model()" neotrix-core/src crates \| grep -v skill_loader.rs \| wc -l` ≥1；`is_official_converged` 计数=0 |
| **⚠️ 同类误判的教训** | 本轮我一度把 `Provenance` 的默认 `ModelAdded` 写成「危险默认」，**读代码后发现作者已明确辩护**（`experience_tree/mod.rs:69-70`：*"默认值必须说真话"*）。⇒ **给别人的代码贴「危险/死/多余」标签前，先读那段代码的注释** —— 这与 R-SCAN-1（扫描器告警须先读现场）是同一条纪律 |

---

## 3. 批次 B —— 测量面（最高长期价值）

### ⬜ B1 · N-12 门可满足性元门（**优先于本批其他项**）

| | |
|---|---|
| **支脉节点** | 全部 `scripts/check-*.sh`（**14 个**，本轮新增 `check-ci-refs.sh`）+ `scripts/ops/`（5 sh + 21 py）+ 全部 CI job |
| **来源教训** | `i-have-adhd` 的发布门**永远无法通过**（"has no blocking findings" 是绝对规则，相邻规则却是比较规则）⇒ *"This is a property of the gate worth deciding on deliberately rather than discovering during a release."* **是跑出来的，不是想出来的** |
| **本仓同源已发生** | A3 的空 baseline（`--strict` 下零容忍）—— **就是一道没人问过可满足性的门** |
| **动作** | 新建 `scripts/check-gate-satisfiable.sh`：对每道 `--strict` 门，**注入已知违规确认它红**（非空门证明）**且**确认存在已知合规输入确认它绿**（可满足性证明） |
| **验收** | 每道门两条证明；**任何一门缺证明即红** |
| **本轮已做的两例** | `check-ci-refs.sh`（注入幻影→红）与 `check-truth-surface.sh`（注入新文件→两类各+1）**都验过** |

### ⬜ B2 · N-11 门记录带 `env` 指纹（`Soup/benchmarks/`）

| | |
|---|---|
| **支脉节点** | `nt_manifest.py:58-79` `env_fingerprint()` = sha256(head+sorted dirty+cargo_lock)[:16] —— **已存在**，只需扩到门/实验记录 |
| **来源机制** | 三层证据：散文 → **可重跑 harness** → per-run JSON 记 `git_sha`。原文：*"an arm that claims to be 'the old code' is only evidence if the JSON says where it came from."* |
| **要抄的具体一条** | `gate-836`：13 次**逐字节相同**配置，config hash/token 数/峰值显存**完全一致**，吞吐横跨 **376.4–915.3 tok/s（2.43×，CV 35.3%）**；噪声分解为**可加**（每 step 近恒定 0.17–0.28 s），并**证伪了自己的两个解释** |
| **验收** | 记录一条实测值；改 `Cargo.lock` 后 `nt_manifest.py stale` 必须 exit 1 |

### ⬜ B3 · 0.2 证伪门（2026-09-28 §0.2，**仍未做**）

| | |
|---|---|
| **为什么最高价值** | 「全清单 11 个自进化仓**无一**证明自己有效」—— 本轮扩到 **30 仓**结论不变，而 kev/Soup 证明**它是可以做到的**。这是唯一能让 NeoTrix **领先而非跟随**的东西 |
| **支脉节点** | `crates/neotrix-audit/`（纯函数、零 LLM）+ 已有门可挂靠 `scripts/ops/` + `.github/workflows/ci.yml` |
| **四个动作** | **A 预注册**：任何「进化」动作前写下接受的结果+证据+目标 commit+固定模型/接口+**会削弱假设的那个结果**<br>**B 四事实证据**：每条被检索的上下文记 `available`/`retrieved`/`invoked`/`relevant` 四个**独立**事实<br>**C 正面+负面都提交**：变好→推进分支，持平/变差→`git reset`，**两种都记进 `results.tsv`**<br>**D 复杂度判据**：0.001 的提升换 20 行 hack → 不要；换「**删掉** 20 行」→ 要 |
| **验收** | 一条单测：改记忆规则而不改证伪门 ⇒ 门红 |
| **⛔ 必须避开的两 shapes** | ① `prime-agent` 的 `AUTO_REFINE_REVIEW_SYSTEM_PROMPT` 只决定「要不要 refine」，**不评估「refine 得好不好」**，而 `RefinementEvent.outcome` 是**模型自己写的自由文本** ⇒ 那是**日志闭环**，不是反馈闭环<br>② `backpass` 只记 "violated" 就删规则 ⇒ **系统性反的**：违反意味着模型**没读到**，不是规则**坏**。删一条规则需 ≥2 会话的 **`harm` 类**负面证据，*"non-compliance never counts, because a rule that was skipped needs reinforcement, not deletion"* |

### ⬜ B4 · 0.3 `maturity_audit` 降级落盘（资产已在，纯接线）

| | |
|---|---|
| **支脉节点** | `crates/nt-core-capability-tree/src/registry.rs:466` `maturity_audit()` · `:485` `demote_mislabeled()`（**自愈已实现**）· `:15-18` `MaturityFinding{claimed,supported}` · `epistemic.rs:23-40` `Epistemic{Exact,LowerBound}`（已接线，`tests/epistemic_wiring.rs` 验证真实路径） |
| **已完成部分** | ✅ **已进 CI 阻塞态**（`ci.yml` `capability-truth` job 调 `audit-maturity --strict`） |
| **缺口** | 门**只报不改**。缺一步：`claimed > supported` 的节点**要么自动下调落盘、要么 CI 失败** |
| **⚠️ 已知隐患** | `.neotrix/capability_overrides.json` 与 `capability_registry.json` **md5 完全相同**（`5e3c6d29f4feceabcbaaa965009c6f3c`）⇒ **overlay 从未分叉，merge 路径在实践中未被测过** |
| **验收** | 人为把某节点 `claimed` 调高 ⇒ 门红并指名 |
| **投入产出比** | 一次调用把 22 轮吸收里所有「声称 C4 但只有 C2」的能力自动降级 |

---

## 4. 批次 C —— 策略面（⚠️ **有并发冲突**）

> **实测（2026-09-29 10:43）**：`crates/neotrix-neobot/src/nt_types.rs` mtime 距当时刻 **36 秒**
> ⇒ **另一窗口正在写这些文件**。本批全部落在 `crates/neotrix-neobot/src/`。
> **动手前必做**：`stat -f "%Sm" crates/neotrix-neobot/src/nt_*.rs` + `git status --porcelain crates/neotrix-neobot/`

### ⬜ C1 · N-6 三态工具策略（`avibe/core/agent_tool_policy.py`，216 行）

| | |
|---|---|
| **支脉节点** | `nt_policy.rs:75-145` `evaluate_policy()`（deny 全集 + default-deny，**方向正确，别动**）· `:225-246` `looks_like_escape()`（**16 词 needle 表，命中即 deny，不给出路**）· `:158-203` `evaluate_extra_deny()` · 调用方 `nt_agent.rs:587` `gate()` → `:819`/`:826` |
| **来源机制** | `allowed=True` 静默 / `allowed=True` **带 advice** / `denied` 带 reason。中间态判词：*"for tools whose background form is legitimate inside a turn but **lossy across one** — a hard block there would cost more than it saves."* 且**每个 deny 指名可复制的替代命令** |
| **动作** | `nt_audit.rs:8-13` `AuditDecision{Allow,Deny}` 扩三态；deny 时必须给替代（若有） |
| **验收** | 16 个 needle 各一 case，标注「真越狱 / 误报」，**误报集非空** ⇒ 证明门可判定 |
| **⛔ 不改** | default-deny 方向是对的。`gryph` 那种 **fail-open 默认**（`Config.FailOpen=true`：检查器出错就放行）是**反面教材** —— 对照 `strands` 的 `HumanInTheLoop`：分类器任何异常 ⇒ 一律「需要批准」 |

### ⬜ C2 · N-7 `StopReason` 与 `TurnStatus` 正交（`strands/types/event_loop.py`）

| | |
|---|---|
| **支脉节点** | `nt_types.rs:27-33` `enum TurnStatus{Done,Continue,NeedsClarification,Blocked,Waiting}` · `nt_agent.rs:434-702` `run_loop()` · `:463` 循环上界 = `config.max_steps.max(1)`（`:450`，**不是 `max_turns`**）· `:698-700` 跑满 → `Waiting` |
| **实测缺陷** | 仓内**无 `StopReason` 类型**。`Waiting` 一次承担两个语义：① **预算耗尽** ② **人可接手** ⇒ 调用方**无法区分「agent 放弃了」与「agent 没机会了」** |
| **来源机制** | 12 个枚举值，三个**资源终态彼此独立**（`limit_turns`/`limit_output_tokens`/`limit_total_tokens`）+ `guardrail_intervened` + `content_filtered` ⇒ 不用解析文本就能区分「agent 选停 / 预算让它停 / 策略让它停」 |
| **动作** | 新增 `StopReason` **正交于** `TurnStatus`；`Waiting` **保留**（人接手是真需求），但**预算耗尽走 `LimitTurns`** |
| **验收** | `max_steps=1` 且模型永不调 `set_turn_status` ⇒ 断言 `StopReason::LimitTurns` **且** `TurnStatus::Waiting`。**两者都要 —— 正交性就是验收点** |
| **⚠️ 验收要防的坑** | 新增门后**必须自问：是否存在任何能通过的实现？**（B1 的教训，`i-have-adhd` 是跑出来才发现的） |

### ⬜ C3–C6（2026-09-28 遗留，实测复核后状态未变）

| # | 任务 | 支脉节点（实测） | 验收 |
|---|---|---|---|
| C3 | 1.4 非不可宽化策略地板 | `nt_policy.rs:75` 现为 deny 全集（`human-control:77`/`computer-allow:84`/`computer-host:90`/`workspace-jail:98,106`/`unknown-tool:119`/`default-deny:121`/`evaluate_extra_deny:137`） | 把「允许绕过批准的能力集合」变成**测试钉死的冻结数据** |
| C4 | 1.1 DNS qtype 白名单 | `egress_types.rs:14-21` **仍只有 `host`/`port`/`allow` 三字段，零 DNS 概念** | ① qtype 白名单（现只管 A/AAAA）② qname 长度上限 ③ `dns_allow` 从「学习提示」改成**真过滤器**<br>⛔ **不要拦** OSINT 的 `dns.rs:74 query_doh`（合法侦察） |
| C5 | 1.2 attempt/outcome 解耦 | `l0_substrate/nt_core_telemetry.rs`（事件 schema）· `l6_meta/nt_safety_monitor.rs` · `nt_core_guardian/health.rs` | 两者成为**独立字段**；任何情况下 outcome 不得衰减 attempt 严重性 |
| C6 | 5.2 MCP capability 协商 | 协议层 `l1_action/nt_act/mcp_protocol/`（775 行，每请求必带，缺字段 `-32602`）；**实测 `agent.rs` + `mcp_server.rs` 命中 `protocolVersion`/`clientCapabilities` = 0** | 客户端发这两个字段；生产入口 `entry/interactive.rs` + `headless.rs` |

---

## 5. 批次 D —— 记忆与检索

> ⛔ **动手前先裁决改哪一份**：`l6_meta/memory/nt_memory_experience_tree.rs`（458 行）
> 是 `experience_tree` 的**同名第二份且完全死**（只有 `mod.rs:33,43` 的声明与 glob re-export）。

| # | 任务 | 支脉节点（实测） | 来源 | 验收 |
|---|---|---|---|---|
| D1 | 2.1 experience_tree supersession | `experience_tree/mod.rs:29-43`（9 字段**只满足 `source`**）`:57-77` `Provenance`（默认 `ModelAdded`）`:481-537` 五阶段 | 照 `nt_temporal_facts.rs:16-32` **已验证**的版本链（每版本独立 id + `supersedes`/`superseded_by`，**绕开主键重写**） | 加 `lifecycle_state` + 指针<br>⚠️ **不要动 `Provenance` 的默认值** —— `:69-70` 作者明确辩护：*"反直觉(多数系统默认"有依据"), 但默认值必须说真话 —— 见 R-SCAN-1"*。那**是如实的默认**（现有条目确是分析产出），⛔ 不是「危险默认」—— 我第一版这么写是误判，**动手前先读那段注释** |
| D2 | 2.2 nodes 表对齐双时间 ⛔ **风险最高** | `l0_substrate/nt_core_kb_primitives.rs:188` **实测仍是 `id TEXT PRIMARY KEY`**（无时间维） | 照 T6 结论：必须用「每版本独立 id」形态，**可完全避开复合主键与外键问题** | ⚠️ 曾试改复合主键**又暴露两处**（`edges` 真外键 `:248-249` 失效；`nodes_as_of()` 文档写「latest version」**实现却返回全部版本**）⇒ **半迁移比不迁移更糟** |
| D3 | N-10 delta-op 记忆更新 | 同 D1 | `hindsight/reflect/delta_ops.py`：**按 id 寻址不按 index**（*"an index must be counted by the model, an off-by-one is still in range and silently overwrites"*，且**没有任何长度收缩检查能抓到它**）；未触及段**物理复制** | 无改动的 refresh 后 `ExperienceEntry` 逐字段相同 |
| D4 | N-9 多因子打分缺失塌中性 | `bm25.rs:3-5`（`K1=1.5,B=0.75,RRF_K=60.0`）`:174 rrf_fuse()` · `nt_pure_fns.rs:403-434 hybrid_search` · `kb_search.rs:401-450 search_permission_aware`（**检索侧唯一权限出口**） | `hindsight/reranking.py:174` `combined = CE × recency × temporal × proof`，有界 ±α/2，**信号缺失恰为 1.0**；另有「透传 reranker ⇒ 从 RRF rank 重新播种」分支 | ① `valid_until` 设过去 ⇒ `temporal_boost`<1.0；② **删除该字段 ⇒ boost 恰为 1.0**（不是 0，不是 NaN）<br>⚠️ `nt_temporal_facts` 的时序信号**已在库但未用** |
| D5 | N-8 缓存键含代码与路径 | `nt_memory_kb/mod.rs:195` `fused_cache` · `kb_search.rs:23-32` key=`format!("search:{}:{}",query,limit)` · `mod.rs:303`+`kb_core.rs:350` 失效=**全量 clear**（无内容哈希）· `nt_core_cache.rs`（580 行 `SemanticCache`，消费方 `gateway/execution.rs:488-585`） | `cocoindex` `hash(inputs)+hash(code)`（**只键入输入的缓存改了 transform 后陈旧产出仍存 = 正确性 bug**）+ `K-Dense` `content_hash()`（**含路径**，重命名/删除改哈希） | 改 `RRF_K` 60→61 后缓存命中率必须掉<br>⚠️ **真值风险低于 A/B/C/D1-D4，别因为好做就提前做** |

---

## 6. 批次 E —— 2026-09-28 遗留（**部分已被证伪或失效**）

### ⬜ E1 · 0.1 注册表收敛（**实测已变**：CapabilityRegistry 已 4→1）

| 类型 | 2026-09-28 文档 | **2026-09-29 实测** | 动作 |
|---|---|---|---|
| `CapabilityRegistry` | 4 份 | **1 份** ✅（`l0_substrate/nt_core_capability_types.rs:533`） | ✅ **已完成**（别的窗口做的） |
| `ToolRegistry` | 3 份 | **3 份**（未变） | ⛔ **不要删 `nt_core_gate` 那份** —— 它有活消费者 `nt_shield_enforcer.rs:388-401`，承载**写操作可逆性**（`reversible`/`irreversible`），与 L1 那份的**运行期计数是正交轴**。自述：*"同一注册表同时发 tool spec 与 gate config, **两者不能分歧**"* |
| `SkillRegistry` | 4 份 | **4 份** | 收敛到 1（正典 = `neotrix-core/src/skill_registry.rs:14`）；⛔ 但先看 A5 的裁决 |

### ⬜ E2 · 阶段 2 桌面执行回路（2.1–2.5）

> ⚠️ **`src-tauri/` 已于 `5c02e738` 归档（599 files）**，本阶段的 GUI 落点需重新裁决。
> 关键机制仍值得吸收：`capture_id`（把坐标绑到它被计算的那一帧，陈旧即拒**且不回落** —— 其余 11 个 GUI 仓的坐标漂移缓解**全为 0**）。

### ⬜ E3 · 阶段 3 成本与上下文（3.1–3.3）

- **3.1 成本归因**：唯一插点已存在（`anthropic.rs:92,237` 请求时组装、从不落 transcript 的不可见前缀）；需把 `skill` 作为一等维度
- **3.2 上下文预算零和算术**（`backpass`）
- **3.3 禁止声明学习加权**（`prime-agent`）

### ⬜ E4 · 5.1 GUI 元素寻址

**实测：`crates/neotrix-neobot/src/nt_computer.rs:98` `NoopBackend` 仍是唯一后端**
（文件头注释自陈 *"当前仅 `NoopBackend`"*）。`l1_action/nt_io/nt_io_desktop/` **只有 2 文件**。

### ⛔ 已被证伪/失效，不再列入

| 原条目 | 状态 |
|---|---|
| **0.4 `UnifiedApi` 脱 stub**（`src-tauri/src/stub.rs:275`） | ⛔ **落点已随 `src-tauri` 归档**（实测 `src-tauri/` 不存在，全仓 `UnifiedApi` 命中 0）⇒ 需重新裁决落点，或直接取消 |
| **0.3 四 orchestrator 择一** | ⛔ 未做；`OrchestratorConfig`/`OrchestratorStats` 等 20 处定义仍在 |
| **2.1 `nodes` 表「从零做双时间」** | ⛔ 方向指错 —— `temporal_facts:41` **已经是真双时间**（`valid_from`/`valid_until` + `created_at`）；`paged_kv`/`kb_kv`/`vector_index` **全是内存结构、无一张表**，「两表」无处落地。**改为 D2** |
| **2.1 supersession「要新建」** | ⛔ 记忆库层早已实现（`nt_memory_curation.rs:182/238` D2 冲突消解 + `tests.rs:564`）⇒ **改为 D1**（范围收窄到 `experience_tree` 自身） |
| **4.1 给决策引擎加 JEV 四件套** | ⛔ **前提证伪**：三个「决策引擎」全无生产消费者（gateway 再导出在无人启用的 optional feature 后；`new()` 仅在 `#[test]` 内；`WeightedScorer`/`DecisionRecommender` 外部引用 0） |
| **5.2 「`McpRegistry` 在 `#[cfg(test)]` 里」** | ⛔ **错**：`agent.rs:496/505` 的 `#[cfg(test)]` 只挂**单个测试辅助项**，registry 在 `pub mod tool{`(420) = 生产面且**已接线**。**真缺口**是客户端不发 capability 字段（实测命中 0）⇒ 保留为 C6，但描述已改 |
| **「有 `evals/` 目录」** | ⛔ **全仓无 `evals/` 目录** |
| **「`.neotrix/context-manifest.json` 是上下文清单」** | ⛔ **它不是** —— 它是 5 条 claims/evidence 记录。仓内**无任何上下文装配清单**（同名不同物） |

---

## 7. ⛔ 需裁决（本清单不给答案，因为不是技术问题）

| # | 事项 | 为什么不能自己决定 |
|---|---|---|
| 1 | **221 个未编译 `.rs` 的逐条处置** | 多数是已归档旧引擎，但归档/补 mod/删是**产品判断**；(c) 类**必须先试编译** |
| 2 | `hybrid_retrieval/` 6 文件 | `mod` 被注释（注「内部编译错误待修复」）⇒ 修编译错误还是删 |
| 3 | `nt_meta/eval_engine/` 641 行 | 接上（**B1/B2/B3 都依赖它**）还是当孤儿 —— 但接线要设计 |
| 4 | `docs/` 要不要恢复 vitepress 站 | `477bf669` 删了 `docs/package.json`，但 `docs/**` **113** 个跟踪文件还在。要么重建，要么把 `paths: docs/**` 触发也删干净 |
| 5 | 阶段 2 桌面回路的落点 | `src-tauri` 已归档 ⇒ 落在 neobot 还是重建 |
| 6 | N-6/N-7 何时做 | 需与正在写 `crates/neotrix-neobot/src/` 的窗口协调 |

---

## 8. 执行前三道闸（每组之间都要重跑）

```bash
# 闸 1：无并发写入 —— 本轮两次踩中（nt_types.rs 36s / book_to_skill.rs 5s）
stat -f "%Sm %N" <要改的文件>
git status --porcelain <要改的文件>

# 闸 2：内存（非 0 禁止起构建）
sh scripts/ops/nt_mem_gate.sh; echo $?

# 闸 3：干净检出可构建（脏树不是合法 oracle —— LESSONS-20260928-fresh-checkout）
git worktree add --detach /tmp/ntx HEAD
```

**回滚**：起点建专用分支，**不要在脏树上直接改**；每组 `git checkout -- <paths>`。

---

## 9. 复核命令（本文所有数字的来源）

```bash
# 已完成项
bash scripts/check-ci-refs.sh --strict; echo $?              # 期望 0
bash scripts/check-truth-surface.sh 2>&1 | sed -n '2,3p'     # 期望 UNDECLARED:0 UNREACHABLE:0

# 状态实测
grep -rn "pub struct CapabilityRegistry" neotrix-core/src crates | wc -l   # 1
grep -rn "pub struct ToolRegistry" neotrix-core/src crates | wc -l         # 3
grep -rn "pub struct SkillRegistry" neotrix-core/src crates | wc -l        # 4
grep -vc '^#' scripts/layer-deps-baseline.txt                            # 101
wc -c scripts/test-failures-baseline.txt                                 # 0  ← A3 的对象
sed -n '14,21p' neotrix-core/src/l1_action/nt_io/nt_io_provider/common/egress_types.rs
grep -c "protocolVersion" neotrix-core/src/agent.rs                      # 0  ← C6 的对象
md5 -q .neotrix/capability_registry.json .neotrix/capability_overrides.json  # 相同 ⇒ B4 的隐患
```

---

## 10. 三条纪律（本轮用代价换来的）

1. **门要问可满足性**（B1）。`i-have-adhd` 的门永远通不过，**是跑出来才发现的**。
   本轮 `check-ci-refs.sh` 第一版也是坏的（`[ -n "$val" ] && return 0` 把「非空」
   当「合法」，**抓到注入的幻影反而放过**）—— 是**非空门证明**抓出来的。
   **没有这步，我会交付一个永远绿的假门**，那正是本轮拆掉的 5 个幻影门的同一种病。
2. **「未被编译」≠「功能缺失」**（A2）。盲删会打断活路径 ——
   `DIR-REMEDY §2.5` 记「导出 ≠ 调用」**已错过 3 次**。
3. **缺失的检索信号塌成 1.0，不是 0**（D4）。塌成 0 会让「没这个信息」变成「最差」。
