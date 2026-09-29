# 核心进化路线（第二轮，2026-09-29）— 支脉节点定位版

> ⚠️ **已被合并取代（同日）**：**排期与状态以
> `EVOLUTION-MAP-CONSOLIDATED-2026-09-29.md` 为准**（它把本文 N-1…N-13 与
> 2026-09-28 的阶段 0–5 合并成单一真源，并补了本轮实测出的状态变化）。
> 本文件保留作 N-1…N-13 的**取证留档**。

> **本文是 `EVOLUTION-ROADMAP-CODE-NODES-2026-09-28.md` 的续篇，不取代它。**
> 2026-09-28 那份的阶段 0（0.1 收敛 / 0.2 证伪门 / 0.3 maturity 接 CI / 0.4 UnifiedApi）
> 的结论本轮全部复核，**0.3 已完成**（`ci.yml` `capability-truth` 阻塞态），
> 0.1/0.2/0.4 未动。本篇只给**本轮新证据**产生的新节点。
>
> **口径纪律（承接 2026-09-28 §0.1，且本轮再次证实）**：
> 「自进化机制本身不是护城河，**能证明自进化是否有效**才是」。
> 本轮把这条从 11 个仓扩到 **30 个仓**，并找到了反解法 —— 见
> `ABSORPTION-AGENT-ARCH2-2026-09-29.md` §1.1。
> **每个节点都带「验收」；没有验收的节点不叫路线，叫愿望。**
>
> **所有锚点 2026-09-29 用 `grep`/`read` 实测复核。** 本篇新增的方法：
> 用**从 crate root 出发的传递可达性**（而非 dep-info）判定「是否被编译」，
> 因为 dep-info 只覆盖单个 target，会把 bin/test target 的文件误判为未编译。

---

# 阶段 0'：先让「绿」有意义（这批不做，后面每一项的验证都不可信）

> 与 2026-09-28 §0 同源，但本轮发现了**三个新的、更靠前的**问题。
> 排序原则：**先让门本身可信，再让代码可信。**

## N-1 🔴 P0 CI 有 4 个 job 指向不存在的目录（门的存在性）

| | |
|---|---|
| **支脉节点** | `.github/workflows/ci.yml:114` `frontend-tests`<br>`:129` `frontend-coverage`<br>`:151` `frontend-build`<br>`:177` `e2e` |
| **实测证据** | `neocodex-frontend/` 在 `.gitignore:136` 被**整目录忽略**，`git ls-files` 跟踪 **0** 个文件；`e2e/` 同样 **0** 跟踪、磁盘不存在。`src-tauri/` 已于 `5c02e738` 归档（599 files），`apps/` 磁盘上不存在 |
| **为什么是 P0** | 这 4 个 job 在**任何干净检出上必然失败**（`npm ci` 在不存在的工作目录）。它们不是"偶尔红"，是**恒红** |
| **更深的问题** | 这就是 `awesome-dsh-plugin` 记的真实事故：*"A gate that dies before posting is indistinguishable from one that never needed to run."* 本仓已经**有**这个前例 —— 提交 `3edf3be7`「发布链路: 移除幻影CSS门禁 (脚本不存在, 构建恒失败)」修的是同一类。**这次是 CI 层复发** |
| **动作** | ① 删掉这 4 个 job（前端已归档，无可测对象）<br>② **加一条自检**：任何 job 的 `working-directory` 指向的路径若不在 `git ls-files` 覆盖范围内 ⇒ 该 job **必须在同一文件里被标注为 `if: false` 并附理由**，否则 `check-ci-refs.sh` 报错 |
| **验收（可执行）** | 新门 `scripts/check-ci-refs.sh`：`exit 0`；且**故意**把一个 job 的 working-directory 改成 `does-not-exist/` 后必须 `exit 1` 并指名该行号（**非空门证明** —— 见 `LESSONS-20260928` 的「非空门回归」） |
| **纪律** | ⛔ 不要为了让 CI 变绿而把目录加进 `.gitignore` 之外或建空目录占位 —— 那是把症状变成谎言 |

## N-2 🔴 P0 212 个 `.rs` 从不被任何 target 编译

| | |
|---|---|
| **支脉节点（门本体）** | `scripts/check-truth-surface.sh:66-86` class 2 UNDECLARED<br>`:69` `find $SCAN_ROOTS -type f -name 'mod.rs' -path '*/tests/*'` ← **限定在字面叫 `tests` 的目录** |
| **支脉节点（受害者，实测抽样）** | `neotrix-core/src/l1_action/nt_act/tool_registry.rs`（**770 行**，`ToolRegistry`/`ToolStats`/`ToolExecutor`；仓内零消费者）<br>`neotrix-core/src/l6_meta/nt_meta/eval_engine/{mod,dataset_manager,experiment_tracker,llm_judge}.rs`（**641 行**，全仓**唯一**的 dataset/experiment/llm-judge 抽象）<br>`neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/{context_budget,process_skill_memory}.rs`<br>`neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/unified_*.rs`（6 个）<br>`neotrix-core/src/neotrix/nt_crystal_core/{observability,self_healing,memory_orchestrator,agent_orchestrator,multi_graph_memory}.rs`<br>`neotrix-core/src/l3_embodiment/nt_shield/defense/**`（21 个，ring_core/ring_inner/ring_outer/ring_boundary）<br>`neotrix-core/src/l5_cognition/nt_mind/nt_mind/{dao_engine,self_evolver,federation,knowledge_miner,ethical_intuition}.rs` |
| **总量** | 实测 **212**（方法：从每个 crate root `lib.rs`/`main.rs`/`bin/*.rs` 出发做**传递 mod 可达性**，含 `#[path=]`/`include!`/内联 `mod x {}` 三种形式；对 2,550 个磁盘 `.rs` 求差集） |
| **实测复核（抽样，全部确认）** | `nt_memory_kb/mod.rs` 无 `mod context_budget`（`grep -c` = **0**）；`nt_act/mod.rs` 无 `mod tool_registry`（**0**）；`nt_meta/mod.rs` 无 `mod eval_engine`（**0**） |
| **为什么是 P0** | `check-truth-surface.sh` 的**动机注释**（`:4-9`）写的正是这件事：*"`cargo test` was fully green while 311 tests … were never compiled … Green signals meant nothing because the offending code was invisible to the toolchain."* —— **同一个病复发在同一个脚本里**，因为修复只覆盖了 `tests/` 目录 |
| **动作** | class 2 的扫描范围从 `-path '*/tests/*'` 改为**全部含 `mod.rs` 的目录**（381 个）；新增 `UNREACHABLE` 类别 = 磁盘上存在但**从任何 crate root 传递不可达** |
| **验收** | `bash scripts/check-truth-surface.sh --strict` 报出 212 条 UNREACHABLE（先入 baseline，**不是**要求一次删完）；**非空门证明**：把某文件临时加 `mod` 声明后该条消失 |
| **⚠️ 判读纪律** | ⛔ **不要**因为"212 个文件没被编译"就推断"212 个功能缺失"。多数是**已归档的旧引擎**（`nt_consciousness_core/archive/`、`ring_defense/` 明显是）。**先分类再处置**：归档 / 补 `mod` / 删除。**盲删会打断活路径** —— 参见 `DIR-REMEDY §2.5`「导出 ≠ 调用已错过 3 次」的同源教训 |
| **交叉发现（同一根因）** | `l4_emotion/nt_memory/hybrid_retrieval/`（6 文件）与 `l6_meta/nt_meta/session_replay/`（12 文件）也在 212 里。前者 `nt_memory/mod.rs:54` 的 `pub mod hybrid_retrieval;` 是**被注释掉的**（注「内部编译错误待修复」）⇒ **它不是"忘了声明"，是"声明了但编译不过"**。这与 `handoff-20260928-consolidated §4.4`「半迁移比不迁移更糟」同源 |

## N-3 🔴 P0 `check-test-baseline.sh` 的账本是空的 ⇒ 「任何失败即红」

| | |
|---|---|
| **支脉节点** | `scripts/check-test-baseline.sh` + `scripts/test-failures-baseline.txt`（**实测 0 字节**，`wc -l` = 0）<br>消费方 `.github/workflows/ci.yml:65` |
| **实测** | `wc -l scripts/truth-surface-baseline.txt` = 22（`test-failures-baseline.txt` 同样 0 字节，`ls -la` 确认） |
| **为什么是 P0** | `handoff-20260928-consolidated §3 T1` 写「账本已无 flaky，技术上可拦」—— **那是以"账本有内容"为前提的**。空账本下 `--strict` 的语义是「**零容忍**」，而 `consolidated §1` 自己记录了「57 条失败（其中 37 条在主工作树已修好、只是从未入库）」。⇒ **现在把 `--strict` 打开，CI 会立刻因 57 条已修但未入库的失败而红** |
| **更深的问题（cline 的教训）** | `cline` 的 eval 框架是本轮见过最完整的（3 层 / `pass@k` **和** `pass^k` / `FLAKY` 一等态 / 失败分类器带 issue 链接），然后 `evals/ARCHITECTURE.md` 自陈 CI 被 `removed`、smoke `disabled`、`benchmarks/tool-precision/DEPRECATED.md`。**测量基础设施被一次重构孤儿化，而没有任何东西会告诉你** |
| **动作** | ① 账本从**干净检出**实测填充（不是主工作树 —— 见 `LESSONS-20260928-fresh-checkout`）<br>② 加**非空门证明**：临时引入一条失败测试 ⇒ 门必须红且指名该测试 |
| **验收** | `bash scripts/check-test-baseline.sh --strict` 在**干净检出**上 `exit 0`；注入失败后 `exit 1` |

## N-4 🟡 P0 `SkillInvocationPolicy` 是门不是核心代码（第一轮误标）

| | |
|---|---|
| **支脉节点** | `neotrix-core/src/skill_loader.rs:107-142` `SkillInvocationPolicy`（`disable-model-invocation` / `user_invocable`） |
| **实测** | `visible_to_model()` / `visible_to_user()` / `is_trusted_only()` **零消费者**（只有 `:123-141` 内部与 tests） |
| **连带证伪** | 同文件 `:197-201` `is_official_converged()` **零调用者** = 死代码<br>`skill_registry.rs:105-170` `route_entity_aware()` **生产零调用**（仅 `:195/:200` 两个测试位）<br>`crates/neotrix-gateway/src/skill_registry.rs:157` 那个 626 行「真典」，唯一调用方 `nt_crystal_serve.rs:1303`，其 `with_defaults()` 指向的两处目录**都不含 `index.json`** ⇒ **它扫不到东西** |
| **动作** | ① `SkillInvocationPolicy` 接到 `SkillLoader::list_skills()` 的返回上（否则那 4 个测试是自证）<br>② 删 `is_official_converged()`<br>③ 裁决 `route_entity_aware` 与 `nt_crystal_serve` 的 registry：**要么接、要么删** |
| **验收** | `grep -rn "visible_to_model()" neotrix-core/src crates src-tauri/src \| grep -v skill_loader.rs \| wc -l` ≥ 1；`is_official_converged` 计数 = 0 |

## N-5 🟢 P0 文档数字 → 证据的可执行追溯（`kev` 的 `verify_claims.py`）

| | |
|---|---|
| **支脉节点（地基已在）** | `scripts/ops/nt_manifest.py:58-79` `env_fingerprint()` = sha256(head + sorted dirty + cargo_lock)[:16]<br>`:96-111` `cmd_add`（claim + evidence[] + env）<br>`:141-158` `cmd_stale`（env 漂移 ⇒ 失效，**exit 1**）<br>`:161-194` `cmd_audit`（查 `file:line` 存在**且行号 ≤ 文件长度**）—— **已挂 pre-commit:64-69，阻塞态** |
| **缺口** | 它验证**引用的位置**，**不验证引用的数字**。`kev/scripts/verify_claims.py` 验证的是后者 |
| **本轮已抓到的实证** | 4 处文档数字已错（见 `ABSORPTION-AGENT-ARCH2` §4）：frontmatter 债 32/59 非 41/58；SKILL.md 68 非 67；`registry.rs:466` 非 `:459`；`check-skill-gate.sh:11` 的 58 应为 59。**全是 `nt_manifest.py` 已能覆盖的类型，只是没人在数字上跑它** |
| **动作** | 扩 `nt_manifest.py`：claim 增加 `printed`（文档里印的字面量）与 `source`（原始结果文件）+ `select`（键过滤）+ `scale`；`audit` 增加「印出的数字能否从 source 按印刷精度复算」<br>**派生量照抄 `kev`**：`macro_mean`（跨路径均值）、`over_requested`（折算回全量，**被拒的算错**） |
| **验收** | 把 §4 的 4 个错数字写进 claims 后 `nt_manifest.py audit` 必须 `exit 1` 并指名 |

---

# 阶段 1：让「策略」可表达（当前是二态 + needle 表）

## N-6 🔴 P1 三态工具策略（`avibe`）

| | |
|---|---|
| **支脉节点** | `crates/neotrix-neobot/src/nt_policy.rs:75-145` `evaluate_policy()`（deny 全集 + default-deny，**方向正确**）<br>`:225-246` `looks_like_escape()` —— **16 个词 needle 表，命中即 deny，不给出路**<br>调用方 `nt_agent.rs:587` `gate()` → `:819` / `:826` |
| **来源机制** | `avibe/core/agent_tool_policy.py`（216 行）：`allowed=True` 静默 / `allowed=True` **带 advice** / `denied` 带 reason。中间态的判词：*"for tools whose background form is legitimate inside a turn but **lossy across one** — a hard block there would cost more than it saves."* 且**每个 deny 指名可复制的替代命令** |
| **为什么重要** | 现在 agent 撞到 deny 就**没有任何出路**。`looks_like_escape` 把「真的越狱」与「命令里恰好含某个词」放在同一分支 |
| **动作** | ① `AuditDecision{Allow, Deny}`（`nt_audit.rs:8-13`）扩为三态<br>② deny 时**必须**给出替代（若有）<br>③ ⛔ **不改** default-deny 方向（它是对的） |
| **验收** | 单元测试钉死：`looks_like_escape` 的 16 个 needle 各自一个 case，标注"真越狱 / 误报"；误报集**非空** ⇒ 证明门是可判定的 |
| **⚠️ 并发警告** | 本轮实测 `nt_types.rs` mtime 距当时刻 **36 秒** ⇒ **另一窗口正在写 neobot**。**接手前先 `stat`** |

## N-7 🔴 P1 `TurnStatus` 把三种停止混成一态（`strands` 的 `StopReason`）

| | |
|---|---|
| **支脉节点** | `crates/neotrix-neobot/src/nt_types.rs:27-33` `enum TurnStatus {Done, Continue, NeedsClarification, Blocked, Waiting}`<br>`nt_agent.rs:434-702` `run_loop()`；`:463` 循环上界 = `config.max_steps.max(1)`（`:450`，**不是 `max_turns`**）<br>`:698-700` 跑满 → `TurnStatus::Waiting` |
| **实测缺陷** | 仓内**无 `StopReason` 类型**（`grep` 零命中）。`Waiting` 一次承担两个语义：<br>① **预算耗尽**（`max_steps` 跑满，模型一直行动不收尾）<br>② **人可接手**（`Blocked`/`NeedsClarification` 也是"人可接手"）<br>⇒ 调用方**无法区分「agent 放弃了」与「agent 没机会了」**。`handoff-20260928-consolidated §3 T7` 说的「预算耗尽要**明确终态**而非悄然截断」就是这个 |
| **来源机制** | `strands/types/event_loop.py` `StopReason`：**12 个枚举值**，其中三个**资源终态彼此独立** —— `limit_turns` / `limit_output_tokens` / `limit_total_tokens`，外加 `guardrail_intervened` 与 `content_filtered`。⇒ 调用方不用解析文本就能区分"agent 选择停"/"预算让它停"/"策略让它停" |
| **动作** | 新增 `StopReason`（正交于 `TurnStatus`）：`EndTurn` / `ToolUse` / `LimitTurns` / `LimitOutputTokens` / `LimitTotalTokens` / `GuardrailIntervened` / `ContentFiltered` / `Cancelled` / `Checkpoint`<br>⚠️ **`Waiting` 保留**（人接手是真需求），但**预算耗尽走 `LimitTurns`**，不再冒充 `Waiting` |
| **验收** | 一条单测：把 `max_steps` 设 1 且模型永不调 `set_turn_status` ⇒ 断言 `StopReason::LimitTurns` **且** `TurnStatus` 仍是 `Waiting`。**两者都要** —— 正交性就是验收点 |
| **⚠️ 验收要防的坑** | `i-have-adhd` 的教训：它的发布门**永远无法通过**（"no blocking findings" 是绝对规则，相邻规则却是比较规则），**是跑出来才发现的**。⇒ **新增门后必须自问：是否存在任何能通过的实现？** |

---

# 阶段 2：让「记忆与检索」不再悄悄陈旧

## N-8 🟡 P2 缓存键必须含代码与路径（`cocoindex` + `K-Dense`）

| | |
|---|---|
| **支脉节点** | `neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/mod.rs:195` `fused_cache: Mutex<LruCache<String, Vec<SearchResult>>>`<br>`kb_search.rs:23-32` key = `format!("search:{}:{}", query, limit)`<br>`mod.rs:303` + `kb_core.rs:350` 失效 = **全量 `fused_cache.lock().clear()`**（**无内容哈希**）<br>`neotrix-core/src/l0_substrate/nt_core_cache.rs`（580 行）`SemanticCache`，消费方 `nt_io_provider/gateway/execution.rs:488-585` |
| **来源机制（两条，互补）** | ① `cocoindex` `@coco.fn`：`hash(inputs) + hash(code)`。原文口径：只键入输入的缓存，**改了 transform 之后陈旧产出仍然存活** ⇒ **这是正确性 bug，不是优化**<br>② `K-Dense` `scan_skills.py:75-95` `content_hash()`：`digest.update(str(relpath))` **在每个文件的字节之前** ⇒ 重命名/删除改哈希，即使存活内容相同 |
| **动作** | ① 缓存键加 **schema 版本 + 代码哈希**（`nt_memory_kb` 的 `bm25.rs` K1/B/RRF_K 常量改动必须失效）<br>② 若引入路径参与，**必须**先确认调用方语义（`query+limit` 是**检索请求身份**，不是产物身份 —— 直接套 K-Dense 的路径哈希会误伤） |
| **验收** | 改 `RRF_K` 60.0 → 61.0 后，缓存命中率必须掉（否则键没含代码） |
| **⚠️ 判读纪律** | 「融合缓存全量 clear」**不是 bug，是保守正确**。它的代价是性能不是真值。**本节点是性能项，优先级低于 N-1/N-2 的真值项** |

## N-9 🟡 P2 多因子打分：信号缺失必须塌成中性（`hindsight`）

| | |
|---|---|
| **支脉节点** | `neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/bm25.rs:3-5`（`K1=1.5, B=0.75, RRF_K=60.0`）`:174` `rrf_fuse()`<br>`nt_pure_fns.rs:403-434` `hybrid_search` = FTS + 语义 的 RRF 融合<br>`kb_search.rs:401-450` `search_permission_aware()` ← **检索侧唯一的权限出口** |
| **现状** | 只有 RRF 融合，**没有** recency / temporal / proof 因子。而 `nt_temporal_facts.rs` 的 `valid_from`/`valid_until` **已经在库里** ⇒ 时序信号**已备但未用** |
| **来源机制** | `hindsight/search/reranking.py:174` `combined = CE_normalized × recency_boost × temporal_boost × proof_count_boost`，每个 boost = `1 + α(x − 0.5)`，**有界 ±α/2**，**信号缺失时该因子恰为 1.0**。另有一段关键防御：显式传入 `is_passthrough_reranker` —— 当配置的 cross-encoder 是恒等透传时，所有归一化分相同，boost 会**成为唯一信号**，把召回静默变成纯时序排序；修法是从 RRF rank 重新播种 |
| **动作** | 加 `temporal` 与 `recency` 两个乘法因子，**缺失塌 1.0**；并加"透传 reranker ⇒ 从 RRF rank 重新播种"的分支 |
| **验收** | 一条单测：把某条记忆的 `valid_until` 设成过去 ⇒ 它的 `temporal_boost` < 1.0；**且**删除该字段 ⇒ boost **恰为 1.0**（不是 0，也不是 NaN） |
| **纪律** | ⛔ 不要把 `search_permission_aware` 的过滤改成排序后过滤（`aliyun §15.8.3`：过滤须在**分页之前**，否则页数泄露不可见资源） |

## N-10 🟢 P2 记忆更新用 delta-op：未触及内容不重新生成（`hindsight`）

| | |
|---|---|
| **支脉节点** | `neotrix-core/src/l5_cognition/nt_mind/nt_mind/experience_tree/mod.rs:29-43` `ExperienceEntry`（9 字段里 tree **只满足 `source`**）<br>`:57-77` `enum Provenance{MaterialBacked, Uncovered, ModelAdded, ExternallyVerified}`，**默认 `ModelAdded`**<br>`:481-537` `ExperienceEngine::run()` 五阶段（消费方 `nt_mind_background_loop/run.rs:631, :713`）<br>**已正确的部分**：`nt_temporal_facts.rs:16-32` 的版本链（每版本独立 id）—— `handoff-20260928-consolidated §3 T5` 说的形态**已经在了** |
| **缺口** | ① 无 `lifecycle_state` / `supersedes`（T5 的真缺口）<br>② **无「未触及内容不重新生成」这条保证** |
| **来源机制** | `hindsight/reflect/delta_ops.py`：LLM 发出**按 id 寻址**的定向操作列表。未触及的段**物理复制**、永不重新生成。零操作 ⇒ 文档逐字节相同。**为什么按 id 不按 index**：*"an index must be counted by the model, an off-by-one is still in range and silently overwrites"*，且**没有任何长度收缩检查能抓到它** |
| **动作** | ① 照 `temporal_facts` 已验证的形态加 `lifecycle_state` + `supersedes`/`superseded_by`（**每版本独立 id，绕开主键重写**）<br>② `Provenance` 默认值 `ModelAdded` 是**危险默认**（未覆盖 = 模型编的）。至少在 `evidence` 为空时**拒绝**写入 |
| **验收** | ① 一次无改动的 refresh 后 `ExperienceEntry` 逐字段相同<br>② 一条测试：`evidence` 空 + `Provenance::ModelAdded` ⇒ 写入被拒 |
| **纪律** | ⚠️ `l6_meta/memory/nt_memory_experience_tree.rs`（458 行）**同名第二份且完全死**（只有 `mod.rs:33,43` 的声明与 glob re-export）。**动手前先确认要改的是哪一份** |

---

# 阶段 3：测量面（2026-09-28 §0.2 的续，最高长期价值）

## N-11 🔴 P1 门记录带 `git_sha` + env 指纹（`Soup`）

| | |
|---|---|
| **支脉节点** | `scripts/ops/nt_manifest.py:58-79` `env_fingerprint()` —— **已存在**<br>`.neotrix/context-manifest.json`（5 条 claims/evidence）<br>消费方 `.githooks/pre-commit:64-69`（阻塞态） |
| **来源机制** | `Soup/benchmarks/`：**三层证据** —— 散文记录 → **可重跑 harness 脚本** → per-run JSON 记 `soup_cli_file` 与 `git_sha`（**它实际导入的那棵树**）。原文：*"an arm that claims to be 'the old code' is only evidence if the JSON says where it came from."* |
| **本轮要抄的具体一条** | `gate-836`：13 次**逐字节相同**的配置，config hash / token 数 / 峰值显存**完全一致**，而吞吐横跨 **376.4–915.3 tok/s（2.43×，CV 35.3%）**。然后它把噪声分解为**可加**（每 step 近恒定 0.17–0.28 s），并**证伪了自己的两个解释** |
| **对 NeoTrix 的直接含义** | 任何「改了 X 所以 Y 变快/变好」的断言，若**没有 env 指纹**，就**不可复现**。`nt_manifest.py` 已有地基，**只需扩到门记录与实验记录** |
| **动作** | `nt_manifest.py add` 扩展到接受 `measurement`（实测值 + 单位 + 噪声地板 + n） |
| **验收** | 记录一条实测值；改动 `Cargo.lock` 后 `nt_manifest.py stale` 必须 `exit 1` |

## N-12 🟡 P1 `assert` 数字的可满足性（`i-have-adhd`）

| | |
|---|---|
| **支脉节点** | 全部 `scripts/check-*.sh`（14 个）+ `scripts/ops/`（5 sh + 21 py）+ 13 个 CI job |
| **来源教训** | `i-have-adhd` 的发布门绝对规则 *"has no blocking findings"*，而相邻规则是比较规则 ⇒ **"阻断项减半（7→3）仍然失败"**。原文：*"as written, no candidate can ever pass while any blocker survives anywhere in the case set … This is a property of the gate worth deciding on deliberately rather than discovering during a release."* |
| **同类已发生在本仓** | N-3 的空 baseline（`--strict` 下零容忍）**就是**这种"没人问过可满足性"的门 |
| **动作** | 新增 `scripts/check-gate-satisfiable.sh`：对每道 `--strict` 门，**注入一个已知违规并确认它红**（非空门证明），同时**确认存在一个已知合规输入并确认它绿**（可满足性证明） |
| **验收** | 14 个门各有一条"注入违规 ⇒ 红"与一条"合规 ⇒ 绿"的证明；**任何一门缺证明即红** |
| **纪律** | 这是**元门**（meta-gate）—— 它检查的是门本身。优先于任何具体门的新增 |

## N-13 🟢 P2 K-Dense 拒绝策略：把「我们不做什么」写进索引

| | |
|---|---|
| **支脉节点** | `skills/index.json`（60 条，10 类别）；`skill_loader.rs:229` `SkillLoader`；`check-skill-gate.sh:38` |
| **来源机制** | `K-Dense-AI/scientific-agent-skills` `AGENTS.md` 的 **"Out of scope, and routinely declined"**：不做通用 SWE skill、不做带科学示例的 infra、不做宽编排器、**不给已能直连的服务加第二 provider**。原文：*"A skill library's biggest failure mode is selection competition, and it is addressed by **refusal policy**, not by quality."* |
| **动作** | `index.json` 加 `out_of_scope[]`；`check-skill-gate.sh` 校验"新增 skill 不落在 out_of_scope 内" |
| **验收** | 一条测试：提交一个落在 `out_of_scope` 的 skill ⇒ 门红并指名命中的那条 |
| **顺带** | `check-skill-gate.sh` 当前 **不在 CI 也不在 Makefile**（实测 `.github/` 与 `Makefile` 零命中），且头注释的 3 个数字全错（见 `ABSORPTION-AGENT-ARCH2` §4）⇒ **先修数字再接 CI** |

---

# 执行顺序与依赖

```
批次 A（真值层，零依赖，可并行）
  N-1 CI 引用自检 + 删 4 个幻影 job
  N-2 truth-surface class 2 扩域 + UNREACHABLE 类
  N-3 test baseline 填充 + 非空门证明
  └─ 这三个做完，"绿"才第一次有意义

批次 B（可证伪层，依赖 A 的绿）
  N-5 claims 数字追溯（扩展已有 nt_manifest.py）
  N-11 门记录带 env 指纹
  N-12 门可满足性元门
  └─ 依赖：要在 CI 已可信的门上加断言

批次 C（策略层，⚠️ 有并发冲突）
  N-4 SkillInvocationPolicy 接线
  N-6 三态工具策略
  N-7 StopReason 正交
  └─ ⚠️ 全部落在 crates/neotrix-neobot/src/，实测有另一窗口在写
     动手前必查 mtime + git status

批次 D（记忆/检索层）
  N-8 缓存键含代码
  N-9 多因子打分缺失塌中性
  N-10 delta-op + Provenance 危险默认
  └─ N-10 前置：先裁决「改哪一份 experience_tree」（有同名死副本）

批次 E（2026-09-28 遗留，未复核）
  阶段 0.1 收敛（4 份 CapabilityRegistry → 2；4 份 SkillRegistry → 1）
  阶段 0.2 证伪门（预注册 / 四事实 / 正面+负面都提交 / 复杂度判据）
  阶段 0.4 UnifiedApi 脱 stub
```

# ⛔ 本轮新增的硬纪律

1. **门要问可满足性**（N-12）。`i-have-adhd` 的门永远通不过，**是跑出来才发现的**。
   本仓已有同类：N-3 的空 baseline。
2. **"未被编译" ≠ "功能缺失"**（N-2）。212 个文件多数是已归档旧引擎。
   **先分类再处置**；盲删会打断活路径（`DIR-REMEDY §2.5` 已错过 3 次）。
3. **"恒红的门"比"没有门"更坏**（N-1）。它训练人忽略红色。
   参照本仓自己的 `3edf3be7`（「移除幻影CSS门禁」）—— 同一类问题在 CI 层复发。
4. **缺失的检索信号塌成 1.0，不是 0**（N-9）。塌成 0 会让"没这个信息"变成"最差"。
5. **缓存不含代码就是正确性 bug**（N-8），不是优化问题 —— 但它的**真值风险低于** N-1/N-2，
   不要因为它好做就提前做。
