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
| 14 | test 失败账本 | 本地 | `check-test-baseline.sh` | ⚠️ **baseline 0 字节**（N-3） |
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
| **N-3** | **test 账本填充** | `cline` 孤儿化教训 | `check-test-baseline.sh` + baseline（**0 字节**） | 干净检出 `--strict`=0；注入失败=1 | ⚠️ |
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
| **N-11** | **门记录带 env 指纹** | `Soup/benchmarks/` | `nt_manifest.py:58-79`（地基已在） | 改 `Cargo.lock` 后 stale=1 | ⬜ |
| **N-12** | **门可满足性元门** | `i-have-adhd`（永远通不过的门） | 新建 `check-gate-satisfiable.sh` | 每门两条证明（红+绿） | ✅ |
> **实测 2026-09-29**：`check-gate-satisfiable.sh` 存在，`.github/workflows/ci.yml` 有引用。
> ⛔ 但注意本仓教训档 `…2026-09-27-scanner-trust`：门记录会腐化，此状态须随改动刷新。
| **0.2** | **证伪门**（预注册/四事实/正负都提交/复杂度判据） | `harness-engineering` 协议 + `autoresearch` | `crates/neotrix-audit/` | 改记忆规则不改门 ⇒ 红 | 🔵 |
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
| **N-6b** | **三态工具策略**（allow/allow+advice/deny） | `avibe/agent_tool_policy.py` | `nt_policy.rs:75-145`（二态）+ `:225-246` 16 词 needle 表 | 误报集**非空** ⇒ 门可判定 | ⬜⚠️ |
| **N-7** | **`StopReason` 与 `TurnStatus` 正交** | `strands/event_loop.py`（12 态） | `nt_types.rs:27-33`（5 态，混了预算耗尽+人接手） | `LimitTurns` **且** `Waiting` | ⬜⚠️ |
| **1.4** | **非不可宽化策略地板** | `ironclaw` | `nt_policy.rs:75`（deny 全集+default-deny） | 「允许绕过批准的集合」**冻结数据** | ⬜⚠️ |
| **1.1** | **DNS qtype 白名单** | `microsandbox` + OpenAI 事故 | `egress_types.rs:14-21`（**仍 3 字段**） | qtype+长度上限+真过滤器<br>⛔ 不拦 OSINT `dns.rs:74` | ⬜ |
| **1.2** | **attempt/outcome 解耦** | OpenAI DNS 事故 | `nt_core_telemetry.rs` | outcome **不得**衰减 attempt | ⬜ |
| **5.2** | **MCP per-request capability 协商** | MCP spec 2026 | `mcp_protocol/` 775 行 ✅；客户端命中 **0** | 客户端发 `protocolVersion` | ⬜ |
| **4.4** | **「模型可见⇒必须已记日志」** | `deepseek-harness` 238k★ | telemetry + audit | 门 | ⬜ |
| **0.5** | **三处同名不同型的第 4 次** | 本地 | 记忆/决策各画**唯一裁决表** | 挂 `check-api-surface.sh` | ⬜ |

## 批次 D · 记忆与检索

> ⛔ **动手前先裁决改哪一份**：`neotrix-core/src/l6_meta/memory/nt_memory_experience_tree.rs`（458 行）
> 是**同名第二份且完全死**（只有 `neotrix-core/src/l6_meta/memory/mod.rs:33,43` 的声明与 glob re-export）。
> **真实活路径**是 `neotrix-core/src/l5_cognition/nt_mind/nt_mind/experience_tree/mod.rs`（1055 行，
> 被 `neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/run.rs:631,713` 消费）。

| ID | 特性 | 来源 | 落点 | 验收 | 状态 |
|---|---|---|---|---|---|
| **1.1m** | **记忆记录的权威头** | `loopx reward-memory` | `experience_tree/mod.rs:29`（9 字段**只满足 `source`**） | `authority`+`confidence` **两个独立枚举**；`Confidence::High` 的 `SoftPreference` 请求写文件⇒拒 | ⬜ |
| **1.2m** | **五个留存标签** | `nanobot` 48.6k★ | `experience_tree` `persist()`+`feedback()` | `[ephemeral]` TTL 到期不在召回集，**内容不含标签** | ⬜ |
| **1.3m** | **delta-ops 取代整体重写** | `Hindsight delta_ops.py` | `distill()`+`persist()` | LLM 返空 op ⇒ 文件 sha256 **不变** | ⬜ |
| **1.4m** | **move-based 退役**（无 embedding 归档） | `Hindsight` | `paged_kv`/`kb_kv`/`vector_index` | 召回热路径少一个谓词 | ⬜ |
| **1.5m** | **记忆层无 LLM** | `memU` 14.4k★ | `nt_mind`（422 文件） | store/embed/retrieve **不做推理** | ⬜ |
| **1.6m** | **两表 checkpoint + ULID** | `langgraph` 42k★ | `nt_store/`（✅ 存在） | prune 中间 checkpoint 后 resume ⇒ `Truncated`，**不是空状态** | ⬜ |
| **2.1** | **experience_tree supersession** | agentmemory | `experience_tree/mod.rs` | 照 `temporal_facts` 已验证形态 | ⬜ |
| **2.2** | **nodes 表对齐双时间** ⛔风险最高 | utopia | `nt_core_kb_primitives.rs:188`（**仍 `id TEXT PRIMARY KEY`**） | 半迁移比不迁移更糟 | ⬜ |
| **4.1** | **负面证据分类** | `backpass` | 记忆写入路径 | `harm` 才可删规则，**`non-compliance` 永不算** | ⬜ |
| **4.2** | **review-due + 三级注意力** | `oh-my-hermes` | `experience_tree` | — | ⬜ |
| **4.0** | **`nt_crystal_core` 是活路径**（纠正既有文档） | `DIR-REMEDY §2.5` | L1 有 6 个消费者 | ⛔ 勿当死代码删 | ✅ 已知 |
| **N-10** | **delta-op 记忆更新** | `Hindsight delta_ops.py` | `experience_tree` | 未触及段**物理复制** | ⬜ |
| **N-9** | **多因子打分缺失塌 1.0** | `Hindsight reranking.py` | `bm25.rs:174`（现只有 RRF） | 删字段⇒boost **恰为 1.0**（不是 0 不是 NaN） | ⬜ |
| **N-8** | **缓存键含代码与路径** | `cocoindex`+`K-Dense` | `kb_search.rs:23-32`（key=query+limit）+ 全量 clear | 改 `RRF_K`⇒命中率掉 | ⬜ 真值风险最低 |

## 批次 E · 桌面执行回路（`src-tauri` 已归档，落点需重裁）

| ID | 特性 | 来源 | 落点 | 验收 | 状态 |
|---|---|---|---|---|---|
| **2.0** | 文档说谎纠正 | 本地 | `ARCHITECTURE.md` §1-§12 已被 §13 推翻 | — | ⬜ |
| **2.1e** | **元素寻址 + 三态身份** | `agent-desktop` Apache-2.0 | `nt_io_desktop/` **只有 2 文件** | a11y 树优先，坐标仅 fallback | ⬜ |
| **2.2e** | **`capture_id`**（坐标绑到计算它的那帧） | `cua-driver` | 同上 | 陈旧即拒**且不回落**（其余 11 仓漂移缓解**全为 0**） | ⬜ |
| **2.3e** | **`VerificationTier` + 8 态终局** | `OpenAdapt` MIT | 同上 | — | ⬜ |
| **2.4e** | **`DeliverySemantics`→派生 `RetryDisposition`** | `agent-desktop` | 同上 | — | ⬜ |
| **2.5e** | **计划级失效门**（15 行） | `nanobrowser` | 同上 | — | ⬜ |
| **3.2** | **上下文预算零和算术** | `backpass` | `apply_context_budget`（已接线） | — | ⬜ |
| **5.1e** | **GUI 元素寻址 B 路** | `Orca`/`artemis` | `nt_computer.rs:98` **`NoopBackend` 唯一后端** | 先承认它是空的 | ⬜ |
| **5.4** | **UI 组件 + 动效 token** | `beautifului`+`transitions.dev` MIT | `src-tauri/frontend`（**已归档**） | SolidJS 移植 1-3h/个 | ⬜ |
| **5.3** | **无 API 重放** | `jev-drone` MIT | `nt_audit.rs`（全仓 `replay` 零命中） | CLI 已有 `audit` 子命令 | ⬜ |
| **1.3** | 严重性校准对 + 职责分离 | `cloudflare/security-audit-skill` MIT | `RUST-STANDARDS.md §17.5` | — | ⬜ |
| **1.7b** | `manifest.json` 矩形契约 | `sprite-gen` Apache-2.0 | 生成资产管线 | — | ⬜ |
| **3.2b** | 声明式能力 manifest + 爆炸半径 | `MangoDisk` **GPL-3.0⚠️借形状** | `.neotrix/capability_registry.json` | 构建期校验 | ⬜ |
| **4.4b** | 覆盖率账本状态机（hunters 不写自己覆盖率） | 本地 | `nt_shield_audit/`（**未接线**） | — | ⬜ |
| **2.6** | 自治循环 git 化 | — | 见 B 批 4.3 | — | ⬜ |

---

## 第 2 部分 · 已证伪/取消（⛔ 15 项，不再列入）

| 特性 | 原出处 | 证伪依据 |
|---|---|---|
| **`UnifiedApi` 脱 stub** | 09-27 建议 1 / 09-28 §0.4 | ⛔ `src-tauri/` 已随 `5c02e738` 归档，**全仓 `UnifiedApi` 命中 0**，落点不存在 |
| **`nt_crystal_core` 是死代码** | `DIR-AUDIT §六` | ⛔ **活路径**，L1 有 6 个消费者 |
| **`nt_jev` 是死代码** | 同上 | ⛔ **真接线的**，5 处消费者 |
| **`neotrix-core/src/l5_cognition/nt_core_gate/nt_tool_registry.rs` 是 stub，删** | 09-28 曾标 ⛔ | ⛔ **有活消费者** `nt_shield_enforcer.rs:388-401`，承载**写操作可逆性**，与 L1 那份**正交** |
| **「从零做真双时间」** | 09-28 路线图 §1.4 | ⛔ `temporal_facts:41` **已经是真双时间**；`paged_kv`/`kb_kv`/`vector_index` **全是内存结构无一张表**，「两表」无处落地 |
| **「supersession 形态要新建」** | 09-28 §2.1 | ⛔ 记忆库层**早已实现**（`nt_memory_curation.rs:182/238` + 91 测试全绿） |
| **JEV 四件套** | 09-28 §4.1 | ⛔ 三个「决策引擎」**零生产消费者** |
| **「`McpRegistry` 在 `#[cfg(test)]` 里」** | 09-28 §5.2 | ⛔ **错**，在 `pub mod tool{`(agent.rs:420) = 生产面且**已接线** |
| **「有 `evals/` 目录」** | 09-28 | ⛔ **全仓无** |
| **「`context-manifest.json` 是上下文清单」** | 09-28 | ⛔ **它是 claims 记录**，仓内无上下文装配清单（同名不同物） |
| **`CapabilityRegistry` 4→1** | 09-28 §0.1 | ✅ **已完成**（实测 1 份，`nt_core_capability_types.rs:533`） |
| **`maturity_audit()` 接 CI** | 09-28 §0.3 / `TODO.md:526` | ✅ **已接且阻塞态**，真实行号 **`registry.rs:466`** 非 `:459` |
| **「`SkillInvocationPolicy` 是核心代码」** | 09-27 | ⛔ **零消费者**，是**门**不是核心代码 |
| **KV 缓存有内容哈希** | — | ⛔ **零命中**，key=query+limit，失效是全量 `clear()` |
| **「关掉 outcome 衰减」= 已有** | 09-28 §1.2 | ⛔ `Provenance` 默认 `ModelAdded` 是**如实的默认**（`:69-70` 辩护），⛔ 不要动 |

---

## 第 3 部分 · 许可证红线（⛔ 决定能抄什么）

| 允许抄代码 | 仅可抄设计 |
|---|---|
| **MIT**：`hindsight` `browser-harness` `anydoc` `PanelUI` `Infographic` `dsh-market` `qc-skills` `K-Dense` `i-have-adhd` `agent-scripts` `avibe` `loopx` `autoresearch`<br>**Apache-2.0**：`harness-sdk` `InsForge` `cocoindex` `OpenShell` `rrsi` `cline` `kev` `Soup` `BugTraceAI` `fw-ai/cookbook` `agentmemory` `utopia` `cua-driver` `agent-desktop` `deer-flow` `memU`<br>**CC0**：`awesome-dsh-plugin`<br>**CC-BY-4.0 + Apache-2.0**：`NVlabs/kda`（文档段 CC-BY） | ⛔ **无 LICENSE**：`jev-dsh-decision` `dshfind` `Hands-On-AI-Engineering`<br>⛔ **闭源不可核**：`supermemory`（引擎）`cue.im` `weco` `glean/waldo` `primeintellect/ramp`<br>⛔ **非 OSI**：`tdeverx/contained-app`（PolyForm NC）`digipulse/GAAI`（ELv2）`multica`（自定义）<br>⛔ **AGPL-3.0**：`volcengine/OpenViking`（只读设计）<br>⚠️ **GPL-3.0**：`MangoDisk`（只借形状） |

---

## 第 4 部分 · ⛔ 需裁决（6 项，本清单不给答案）

| # | 事项 | 为什么不能自己决定 |
|---|---|---|
| 1 | **212 条未编译 `.rs` 的逐条处置** | 归档/补 mod/删是**产品判断**；(c) 类（`mod` 被注释）**加回去会让干净检出编不过** |
| 2 | `hybrid_retrieval/` 6 文件 | `neotrix-core/src/l4_emotion/nt_memory/mod.rs:54` 注「内部编译错误待修复」⇒ 修还是删 |
| 3 | `nt_meta/eval_engine/` 641 行 | 接上（B 批全依赖它）还是当孤儿 —— 接线要设计 |
| 4 | `docs/` 恢复 vitepress 站 | 113 跟踪文件在，`package.json` 已删 |
| 5 | 批次 E 桌面回路落点 | `src-tauri` 已归档 ⇒ 落 neobot 还是重建 |
| 6 | N-6b/N-7 何时做 | 需与写 `crates/neotrix-neobot/src/` 的窗口协调 |

---

## 第 5 部分 · 执行前三道闸

```bash
# 闸 1：无并发写入 —— 本轮两次踩中（nt_types.rs 36s / book_to_skill.rs 5s）
stat -f "%Sm %N" <要改的文件>; git status --porcelain <要改的文件>
# 闸 2：内存（非 0 禁止起构建）
sh scripts/ops/nt_mem_gate.sh; echo $?
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
