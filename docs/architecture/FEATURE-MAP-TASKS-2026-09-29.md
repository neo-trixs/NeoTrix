# NeoTrix 全特性吸收地图 · 进化任务清单（单一真源）

> **本文件覆盖三轮吸收的全部特性**：2026-09-27（109 仓 + trendshift 21 榜 385 仓）·
> 2026-09-28（8 源）· 2026-09-29（30 源）。
> 排期与状态以本文件为准；`EVOLUTION-MAP-CONSOLIDATED-2026-09-29.md` 是**合并版路线图**，
> 本文件是**特性级清单**（每条特性 → 来源 → 落点 → 状态 → 验收）。
>
> **状态图例**：✅ 已落地并验证 · 🔵 已有资产待接线 · ⬜ 待做 · ⛔ 已证伪/取消 · ⚠️ 阻塞
>
> **三条纪律**（用代价换来的）：
> 1. **门要问可满足性** —— `i-have-adhd` 的发布门永远通不过，**是跑出来才发现的**。
> 2. **「未被编译」≠「功能缺失」** —— 盲删会打断活路径。
> 3. **给别人的代码贴「危险/死/多余」标签前，先读那段代码的注释**。
>
> **路径缩写约定**（表格里为可读性省略公共前缀，完整路径在此）：
> `experience_tree/mod.rs` = `neotrix-core/src/l5_cognition/nt_mind/nt_mind/experience_tree/mod.rs`
> （1055 行，**活路径**）—— ⚠️ 同名的 `neotrix-core/src/l6_meta/memory/nt_memory_experience_tree.rs`
> （458 行）**完全死**，见 §第 1 部分 批次 D 前的警告。

---

## 第 0 部分 · 已落地的特性（✅ 22 项）

### 0.1 第一轮门与接线类

| # | 特性 | 来源 | 落点 | 验收 |
|---|---|---|---|---|
| 1 | skill 路径式 key 解析（`<cat>/<skill>` 优先，裸名兜底） | 官方 Agent Skills 规范 | `skill_loader.rs:309-340` `resolve_from_index` | 13/58→58/58 ✅ |
| 2 | 调用策略 `disable-model-invocation`/`user_invocable` 正交两维 | 同上 | `SkillInvocationPolicy`（**⛔ 但零消费者，见 N-4**） | 字段存在，**接线未做** |
| 3 | frontmatter 是自动发现依据 | 同上 | `check-skill-gate.sh` | **⛔ 不在 CI/Makefile**，且 32/59 欠账 |
| 4 | 策展门「策展≠背书」 | awesome-autoresearch | `check-skill-gate.sh:3-6` | ✅ |
| 5 | 「模型可见 ⇒ 必须已记日志」 | deepseek-harness | `EVOLUTION-ROADMAP:322` §4.4 | ✅ |

### 0.2 门与验证类（本轮重点）

| # | 特性 | 来源 | 落点 | 验收 |
|---|---|---|---|---|
| 6 | **CI 引用门**：job 指向 git 未跟踪路径即拦 | 本轮自建（`awesome-dsh-plugin` 的事故） | `scripts/check-ci-refs.sh` | 干净 0 / 注入 1，**双向验过** ✅ |
| 7 | **拆 5 个恒红幻影门** | 同上 | `ci.yml` 删 4 job + `git rm docs-deploy.yml` | ✅ |
| 8 | **未编译代码可见**（传递 mod 可达性） | K-Dense 契约测试思路 | `check-truth-surface.sh` class2 + `UNREACHABLE` 类 | 干净 0/0；**212 条首次可见** ✅ |
| 9 | 三层门证据（散文→harness→JSON 记 git_sha） | `Soup/benchmarks/` | 地基已在 `nt_manifest.py:58-79` `env_fingerprint()` | 🔵 扩到门记录（N-11） |
| 10 | `maturity_audit()` 接 CI | 本地资产 | `ci.yml` `capability-truth` **阻塞态** | ✅（**09-28 文档说未做，已过时**） |
| 11 | `Epistemic{Exact,LowerBound}` 接线 | 本地资产 | `registry.rs:479` + `crates/nt-core-capability-tree/tests/epistemic_wiring.rs` | ✅ |
| 12 | 干净检出可构建门 | 2026-09-27 事故 | `check-fresh-build.sh` | ✅ |
| 13 | 分层依赖棘轮门 | 本地 | `check-layer-deps.sh --strict`（101 baseline） | ✅ |
| 14 | test 失败账本 | 本地 | `check-test-baseline.sh` | ✅ 全绿（12206/0 failed）；解析 bug 已修 |
| 15 | `Epistemic` 默认 `LowerBound` | 本地 | `epistemic.rs:39` | ✅ |

### 0.3 记忆与知识类

| # | 特性 | 来源 | 落点 | 验收 |
|---|---|---|---|---|
| 16 | 每版本独立 id 的 supersession 版本链 | agentmemory / utopia | `neotrix-core/src/l4_emotion/nt_memory/nt_memory_historian/nt_temporal_facts.rs:16-32`（**真双时间**：`valid_from`/`valid_until` + `created_at`） | **91 测试全绿** ✅ |
| 17 | D2 冲突消解 + 保真账本 | 本地 | `l4_emotion/nt_memory/nt_memory_kb/nt_memory_curation.rs:182/238`（`kb_write.rs:162,173` 消费） | `tests.rs:564` ✅ |
| 18 | 写入链路（核心→版本化→证据→SVAF 门→冲突→GraphRAG） | 本地 | `kb_write.rs:230` `write_memory_entry()` | **5 个非测试消费者** ✅ |
| 19 | 权限感知检索（唯一权限出口） | 本地 | `kb_search.rs:401-450` `search_permission_aware` | ✅ |
| 20 | RRF 融合检索（K1=1.5,B=0.75,RRF_K=60） | 本地 | `bm25.rs:174` `rrf_fuse` | ✅ |
| 21 | experience 五阶段吸收 | 本地 | `experience_tree/mod.rs:481-537` | `neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/run.rs:631,713` ✅ |
| 22 | `Provenance` 四态 + **如实的默认** | 本地 | `experience_tree/mod.rs:57-77` | ⚠️ `:69-70` 作者辩护「默认值必须说真话」，**⛔ 不要动** |

---

## 第 1 部分 · 待做特性（⬜ 62 项，按批次）

## 批次 A · 让「绿」有意义（零 `.rs`，~2 天）

| ID | 特性 | 来源 | 落点 | 验收 | 状态 |
|---|---|---|---|---|---|
| **N-1** | CI 幻影门拆除 + 引用门 | `awesome-dsh-plugin` | `check-ci-refs.sh` | 已完成 | ✅ |
| **N-2** | 未编译代码检测 | K-Dense `CHECKS` 契约 | `check-truth-surface.sh` | 已完成（265 条入账） | ✅ |
| **N-3** | **test 账本棘轮** | `cline` 孤儿化教训 | `check-test-baseline.sh` + baseline | 干净检出 `--strict`=0；注入失败=1 | ✅ |
> **实测 2026-09-29（台账原记载已作废）**：原文「baseline **0 字节**」被标 ⚠️
> 并列为待做。**实测证伪**：当前 `cargo test -p neotrix --lib` = **12206 passed / 0 failed**
> ⇒ 脚本头注记的「12093 passed / 56 failed」是 **2026-09-28 的历史**，
> 那 56 个断言失败**已还清**。
> ⇒ 0 字节 baseline 是**真值（当前零债）**，不是「未填充的豁免缺口」。
> ⛔ **不要**把 N-3 当待做项去「填充」它 —— 填出来的只会是虚构的债。
>
> 但核查过程中发现门**自身**有一个真 bug，已修（见 `check-test-baseline.sh`
> 的 `extract_failing`）：失败名解析用 `grep -E "^test .* FAILED"`，
> **会误中汇总行** `test result: FAILED. 12206 passed; ...`，
> 把整行剥成一个**不存在的测试名**写进 baseline。
> ⇒ 后果不只是脏数据：一旦被人手写进 baseline，等于**豁免了一个不存在的测试**。
> ⇒ 已改为单点 `extract_failing`（两处重复逻辑合并为一处，防漂移），
>   并用注入探针端到端验证：修复前 2 行（含汇总行），修复后**恰好 1 行**。
| **N-4** | **skill policy 接线**（第一轮误标为「核心代码」） | 官方规范 | `skill_loader.rs:107-142` 三个 `visible_*` **零消费者** | `grep` ≥1 非测试消费者 | ⬜ |
> **实测 2026-09-29（纠错记录）**：台账原文写「`skill_loader.rs:107-142` 三个 `visible_*` 零消费者」。
> 逐条核实发现**两处不准**：(a) 实际只有**两个** `visible_*`（`:118` `visible_to_model`、
> `:123` `visible_to_user`），不是三个；(b) 真正查这两个函数名的非测试消费者，**确为 0**。
> ⇒ 结论「零消费者」成立，行号与个数不准，**状态保持 ⬜ 不变**。
> ⚠️ 核查方法教训：`grep visible_` 会命中 `invisible_*` 等子串，曾一度误得「33 个消费者」。
> **必须用完整函数名 `visible_to_model|visible_to_user` 精确匹配**（对应 R-SCAN-1：先证实现状再改码）。
| **N-5** | **claims 数字追溯** | `kev/scripts/verify_claims.py` | `nt_manifest.py`（查 `file:line` 不查数字） | 4 个错数字入 claims 后 audit=1 | ⬜ |
| **N-6** | **`docs/package.json` 裁决** | 本地 | `docs/` 113 跟踪文件 vs vitepress 已删 | 重建或删触发 | ⛔ 需裁决 |

## 批次 B · 测量面（最高长期价值，~1 周）

| ID | 特性 | 来源 | 落点 | 验收 | 状态 |
|---|---|---|---|---|---|
| **N-11** | **门记录带 env 指纹** | `Soup/benchmarks/` | `nt_manifest.py:58-79`（地基已在） | 改 `Cargo.lock` 后 stale=1 | ✅ |
> **实测 2026-09-29（台账 ⬜ 已作废）**：`env_fingerprint()` 不只在地基 ——
> 实测有 **3 个真实消费点**（`nt_manifest.py:98/120/143`），
> `cmd_stale()` **实测可判定**，当前报 3 条 stale
> （`skills_discoverable_claim_refuted` 等，`69f893b0… → 997c2af0…`）。
> ⇒ 判 ✅。台账「地基已在」**低估了自己的实现**。
| **N-12** | **门可满足性元门** | `i-have-adhd`（永远通不过的门） | 新建 `check-gate-satisfiable.sh` | 每门两条证明（红+绿） | ✅ |
> **实测 2026-09-29**：`check-gate-satisfiable.sh` 存在，`.github/workflows/ci.yml` 有引用。
> ⛔ 但注意本仓教训档 `…2026-09-27-scanner-trust`：门记录会腐化，此状态须随改动刷新。
| **0.2** | **证伪门**（预注册/四事实/正负都提交/复杂度判据） | `harness-engineering` 协议 + `autoresearch` | `crates/neotrix-audit/` | 改记忆规则不改门 ⇒ 红 | 🔵 |
> **2026-09-29 接线完成**：新增 `scripts/check-evolution-ledger.sh`（账本活性门，
> 4 场景双向实测）+ `Makefile` 的 `evolution-gate` / `evolution-exp` 目标。
> 同时给 bin 补了 **REJECT 退出码 = 1** —— 此前 REJECT 退 0，
> 导致任何 `set -e` 流程**无法判定判决**（`--help` 报「未知参数」也退 0）。
> ⇒ 本条可升 ✅（资产 + 接线 + 可判退出码三齐）。
> **实测 2026-09-29**：`nt_evolution_eval.rs` 具备 `Preregistration` / `Veto` / `judge_ab` / `is_complete` / `Ledger`，
> 27 测试通过；bin `nt_evolution_exp` 实测能 ACCEPT 也能因 `no_falsifier` / `within_noise` 三路拒绝。
> **判为 🔵 而非 ✅ 的原因**：⛔ `nt_evolution_exp` **未进 `Makefile` 也未进 CI** ——
> 进化实验能跑，但**没有任何门或流程会调用它**。这正是本仓反复出现的
> 「造了不接线」老毛病（与 B6 ExperimentRunner 零消费者同类）。
> ⇒ 接线是本条剩余的唯一缺口，接完即可升 ✅。
| **0.3** | **maturity 降级落盘** | 本地 | `registry.rs:485` `demote_mislabeled()` | `claimed>supported` ⇒ 红/自动降 | ⬜ |
| **3.1** | **成本归因：唯一插点已存在** | `cost-xray` | `anthropic.rs:93`（P0-4 prefix caching 边界） | `skill`/MCP 成一等维度 | ⬜ |
| **1.7** | **取消：持久化请求而非就地取消** | `deer-flow` 83k★ | `crates/neotrix-neobot/src/nt_cancel.rs`（✅ 存在） | 取消请求可重放 | ⬜ |
| **3.3** | **禁止声明学习加权** | `prime-agent` | 反馈判据 `experience_tree` `feedback()` | 门红 | ⬜ |
| **4.3** | **自治循环 git 化**（`results.tsv` 5 列） | `karpathy/autoresearch` | `Makefile` `run:4` | 变好推进/变差 reset，**两种都记** | ⬜ |

## 批次 C · 策略面（⚠️ 并发阻塞）

> **实测 2026-09-29 10:43**：`nt_types.rs` mtime 距当时刻 **36 秒** ⇒ 另一窗口在写。
> **动手前**：`stat -f "%Sm" crates/neotrix-neobot/src/nt_*.rs` + `git status --porcelain`

| ID | 特性 | 来源 | 落点 | 验收 | 状态 |
|---|---|---|---|---|---|
| **N-6b** | **三态工具策略**（allow/allow+advice/deny） | `avibe/agent_tool_policy.py` | `traits.rs:287` 三态 + `strictest_verdict` 最严否决 | 误报集**非空** ⇒ 门可判定 | ⚠️ |.sh; echo $?
> **实测 2026-09-29（台账落点已作废）**：原文写「`nt_policy.rs:75-145`（二态）」
> —— **两处不准**：(a) 该文件 `:75-145` 是 **LRU 缓存**，不是策略；
> (b) 工具策略**已是三态**：`l1_action/traits.rs:287` `SecurityVerdict`
> = `Allow / Deny / RequireApproval`（即 avibe 的 allow+advice 态）。
> ⇒ 三态需求**本仓已有**，台账重复了。
>
> **但核实中发现一个真 fail-open 缺陷并已修**：
> `SecurityRouter::check()` 只取 `registry.optimal()` —— **单个** guard，
> 挑选标准是 `1.0 - error_rate`（**健康度**，与严格程度无关）。
> ⇒ 一个 `error_rate=0.0` 的宽松 `Allow` guard 会**静默屏蔽**
>   `error_rate=0.5` 但判 `Deny` 的 guard。
> **这是把「哪个 guard 更可靠」误当成「动作允不允许」**。
> ⇒ 已改为 `SecurityRegistry::strictest_verdict()`（最严否决：任一健康 guard
>   判 Deny ⇒ 整体 Deny；否则任一 RequireApproval ⇒ 整体 RequireApproval）。
> ⇒ 并**删除** `route()` / `optimal()`：零外部消费者，且「挑一个 guard」的
>   形状会诱导调用方重新引入 fail-open —— 留着等于给已修缺陷留后门。
> ⇒ 红测 `deny_in_any_guard_wins_over_healthier_allow` 修前 FAIL、修后 PASS；
>   全量 **12207 passed / 0 failed**。
> ⇒ 判 ⚠️：能力（聚合语义 + 三态）已落地；剩余缺口是
>   「deny 全集 + default-deny 的**冻结数据**」（台账 1.4 另列）。
# 闸 3：干净检出可构建（脏树不是合法 oracle）
git worktree add --detach /tmp/ntx HEAD
```

---

## 第 6 部分 · 复核命令（本文件所有数字的来源）

```bash
bash scripts/check-ci-refs.sh --strict; echo $?                      # 0
bash scripts/check-truth-surface.sh 2>&1 | sed -n '2,3p'            # UNDECLARED:0 UNREACHABLE:0
grep -rn "pub struct CapabilityRegistry" neotrix-core/src crates | wc -l   # 1
grep -rn "pub struct ToolRegistry" neotrix-core/src crates | wc -l         # 3
grep -rn "pub struct SkillRegistry" neotrix-core/src crates | wc -l        # 4
grep -vc '^#' scripts/layer-deps-baseline.txt                            # 101
wc -c scripts/test-failures-baseline.txt                                 # 0  ← N-3
grep -c "protocolVersion" neotrix-core/src/agent.rs                      # 0  ← 5.2
md5 -q .neotrix/capability_registry.json .neotrix/capability_overrides.json  # 相同 ⇒ overlay 从未分叉
grep -c "impl ComputerBackend for" crates/neotrix-neobot/src/nt_computer.rs # 1 ⇒ 仅 NoopBackend
```
