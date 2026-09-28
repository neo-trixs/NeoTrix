# 核心进化路线 — 2026-09-28 正典（支脉节点定位版）

> **本文取代 `EVOLUTION-ROADMAP-CODE-NODES-2026-09-27.md`**，后者归档至 `_superseded/`。
> 吸收源：**109 个指定仓 + trendshift.io 21 个榜单 385 仓 + 5 篇 arXiv**（去重 474 仓）。
> 全部锚点 2026-09-28 用 `grep`/`read` 实测复核，**非文档转述**。
> 配套：`DIR-REMEDY-2026-09-28.md`（目录架构解法，本文只给节点，不给搬砖顺序）。

---

## 0. 本轮吸收的**方法论级**发现（比任何单个机制都重要）

### 0.1 🔴 全行业**没有一个人证明过自己的自进化有效**

我把 11 个「自进化 / self-improving」仓逐一检查了「有没有一个仓内评测在测量它自己」：

| 仓 | 自进化主张 | 仓内有测量吗 |
|---|---|---|
| **prime-agent** (21k) | Continual Harness `/refine` | **无**。`scripts/evals/short_swe/harness-baselines.json` 比的是**静态** harness（`pi-coding-harness` 22/28），全树**没有 `/refine` 开 vs 关的两臂对照** |
| **backpass** (1.2k) | 对 AGENTS.md 做梯度下降 | **无**。自承：*"Causal attribution is genuinely hard. A model can confabulate influence."* |
| **loopx** (6k) | Reward Memory | **半个**。`loopx reward-memory evaluate` 是 8 例**确定性契约**测试，明说"**does not claim semantic uplift**" |
| **oh-my-hermes** (3k) | 记忆 + 技能 | **无**。README：*"No measured run has been published yet."* |
| **harness-engineering** (2.7k) | 方法论 | **零测量**。`evals/README.md` 是**协议**，一个数都没有 |
| **hermes-agent** (250k) | 轮后后台复盘 | **完全无门** |

**⇒ 对 NeoTrix 的直接含义**：自进化机制本身**不是**护城河，**「能证明自进化是否有效」才是**。
NeoTrix 若把 4.x 阶段的自进化接上，**同时**接上一道 0.2 的证伪门，就跑赢这 11 个仓的全部。

### 0.2 ⛔ 反面教材：prime-agent 的形状必须避开

它的 `AUTO_REFINE_REVIEW_SYSTEM_PROMPT` 是一个 LLM 复审门，**它只决定「要不要 refine」，不评估「refine 得好不好」**。
而 `RefinementEvent.outcome` 字段是**模型自己给自己写的自由文本**。

> 这不是反馈闭环，是**日志闭环**（journaling loop）。

### 0.3 ⛔ 另一个必须避开的：只记 "violated" 就删规则

**backpass** 的核心洞察（README §3）：

> 每个负面证据带一个**类别**：`harm` | `non-compliance` | `irrelevant` ——
> *"those argue for opposite fates: **harm argues against an instruction, non-compliance argues for reinforcing it**."*
>
> 且**非对称删除规则**：删一条规则需要 ≥2 个会话的 **`harm` 类** 负面证据 ——
> *"non-compliance never counts, because a rule that was skipped needs reinforcement, not deletion."*

⇒ **任何把「规则被违反」当作「规则该删」的记忆系统，都是系统性反的。**
违反意味着模型**没读到**，不是规则**坏**。这条落在 `experience_tree` 的记忆写入路径上（见 2.1）。

---

# 阶段 0：把「声称」变成「已证」（1-2 天，不依赖任何外部吸收）

> 阶段 0 全部是**接线 / 删 stub / 建门**。不做完，后面每一项的验证都不可信。
> **实测状态（2026-09-28）：0.1 仍是 stub，0.2 仍未收敛。** 详见 §执行状态。

## 0.1 ⛔ 4 份 `CapabilityRegistry` + 3 份 `ToolRegistry` + 4 份 `SkillRegistry` 收敛

| | |
|---|---|
| **支脉节点（4 份 `CapabilityRegistry`，实测仍 4）** | ① `neotrix-core/src/l0_substrate/nt_core_capability_types.rs:533`（`HashMap<Arc<dyn>>`+by_domain/by_layer）<br>② `neotrix-core/src/l5_cognition/nt_core/capability/registry.rs:462`（`Vec<Capability>`+tag_index，**仅自测**）<br>③ `neotrix-core/src/neotrix/nt_file_ability/capability.rs:185`（精简版，**零消费者**）<br>④ `crates/nt-core-capability-tree/src/registry.rs:60`（`IndexMap`+experience_targets，**真典**） |
| **3 份 `ToolRegistry`** | ① `neotrix-core/src/l1_action/nt_act/tool_registry.rs:85` ← **真典**（运行期 `ToolStats`）<br>② `neotrix-core/src/l5_cognition/nt_core_gate/nt_tool_registry.rs:11` ← 🔴 **⛔ NOT a stub，见下方证伪 —— 禁止删除**<br>③ `neotrix-core/src/l2_perception/nt_world/crawl/agentic_browse.rs:34`（`Vec<_ToolAction>`，crawl 域自包含 `:65/:302/:318/:331/:410`） |

> ### 🔴 0.1 修正（2026-09-28 实测证伪）—— **本项曾被标为「45 行 stub，建议整文件删除」，是错的**
>
> `nt_core_gate/nt_tool_registry.rs` **有活消费者**：
> `neotrix-core/src/l3_embodiment/nt_shield_enforcer.rs:388-390`
> ```rust
> fn write_action_registry() -> &'static crate::l5_cognition::nt_core_gate::ToolRegistry {
>     static REG: LazyLock<…> = LazyLock::new(|| ToolRegistry::new()
>         .register(ToolSpec::reversible("write_file", "undo_file"))
>         .register(ToolSpec::irreversible("git_force_push"))  /* … */
> ```
> 它承载的是**写操作可逆性**（`reversible`/`irreversible`），与 ① 的**运行期计数**是**正交轴**。
> 自述注释：*"同一注册表同时发 tool spec 与 gate config, **两者不能分歧**"*。
> **删它 = 打断 shield enforcer 的写操作单向事实源。**
> ⇒ 改为：加防误删注释（B-4），**不删**。③ 同理不动。
| **4 份 `SkillRegistry`** | ① `neotrix-core/src/skill_registry.rs:14`（L3 门面）<br>② `crates/neotrix-types/src/core/skill.rs:54` ③ `.../core/skills/mod.rs:25`（**包内自重复，零风险**）<br>④ `crates/neotrix-gateway/src/skill_registry.rs:157`（真典，被 `nt_crystal_serve` 用） |
| **动作** | 🔴 **取消「删 `nt_core_gate`」与「③ 合并」两条**（见上方证伪）。**只删 2 份确认零消费者的 `CapabilityRegistry`**：`neotrix/nt_file_ability/capability.rs:185` + `l5_cognition/nt_core/capability/registry.rs:462` ⇒ **4 → 2**。 |
| **验收** | `grep -rn "pub struct CapabilityRegistry" neotrix-core/src crates src-tauri/src \| wc -l` == 1（当前 **4**）<br>同理 `ToolRegistry` **保持 3**（正交，见上方证伪）、`SkillRegistry` == 1（当前 **4**） |
| **纪律** | 收敛期间禁止新增第 5 份。挂 CI 断言。 |

## 0.2 🟡 证伪门 —— **本轮最高价值新增项**

> 借鉴 `harness-engineering/evals/README.md`（CC-BY-4.0，**只引协议不抄散文**）+ `backpass` 的机械门。
> **这是 11 个仓全都没有的东西。**

| | |
|---|---|
| **支脉节点** | 落点：`crates/neotrix-audit/`（纯函数、零 LLM）+ `scripts/test-failures-baseline.txt`（账本已在跑）<br>已有门可挂靠：`scripts/ops/`（15 门）+ `.github/workflows/ci.yml`（13 job） |
| **动作 A · 预注册** | 任何「进化」动作前写下：接受的结果 + 证据 + 目标 commit + 固定的模型/接口 + **会削弱假设的那个结果** |
| **动作 B · 四事实证据** | 每条被检索的上下文记四个**独立**事实：`available` / `retrieved` / `invoked` / `relevant`<br>*（原文：*"An offered context bundle estimates the effect of making context available. It estimates the effect of read context only when the worker actually retrieves it."*）* |
| **动作 C · 正面 + 负面都提交** | 借 `autoresearch`（MIT，96.9k★）：变好→推进分支，持平/变差→`git reset`，**两种都记进 `results.tsv`（tab 分隔）**<br>原文：诚实的负面结果进仓 *"is a good signal, not an embarrassment"* |
| **动作 D · 复杂度判据** | 0.001 的提升换 20 行 hack → 不要；换「**删掉** 20 行」→ 要 |
| **验收** | 一条单测：改记忆规则而不改证伪门 → 门红 |
| **为什么放阶段 0** | 阶段 4 的自进化全部要过这道门。**门后建，上层就是无门裸奔。** |

## 0.3 🟢 `maturity_audit()` + `Epistemic` 接 CI —— **已有资产，纯接线**

| | |
|---|---|
| **支脉节点** | `crates/nt-core-capability-tree/src/registry.rs:466` `maturity_audit()`<br>`:488` 自愈调用 · `:15-18` `MaturityFinding { claimed, supported }`<br>**已扩展**：`registry.rs` 有 `Epistemic` 接线（`tests/epistemic_wiring.rs` 验证「确实接进**真实路径**」）<br>CI：`ci.yml:214` `capability-truth` job（**已阻塞态** ✅） |
| **缺口** | `capability-truth` 门在跑，但**没有把「声称 > 证据」自动降级落盘**。门只报，不改。 |
| **动作** | 门内加一步：跑 `maturity_audit()`，`claimed > supported` 的节点**要么自动下调落盘、要么 CI 失败** |
| **投入产出比** | 一次调用把 22 轮吸收里所有「声称 C4 但只有 C2」的能力自动降级 |

## 0.4 🔵 `UnifiedApi` 脱 stub —— 全项目单点收益最大（**仍未做**）

| | |
|---|---|
| **支脉节点** | `src-tauri/src/stub.rs:275` `pub struct UnifiedApiImpl;` ← **零状态单元结构体**<br>`:260-273` trait 6 方法 · `:284-304` `impl`，`:289` 返回**字面量**<br>`src-tauri/src/main.rs:52` 导入 · `:387` `:407` 构造（system_state 轮询 + 对话） |
| **关键发现（降低改造量）** | `UnifiedResponse.metadata`（`stub.rs:292-303`）**已预留** `layers_involved` · `capabilities_used` · `consciousness_state{phi,coherence,gwt_resonance,emotion,attention_focus}` · `confidence: f64`<br>→ **契约已定好，只差填值**，与 L5 意识核类型级吻合 |
| **动作** | ① `UnifiedApiImpl` 持状态（`Arc<CrystalCore>`，见下方 4.0 的接线说明）<br>② `handle()` 委派真实对话路径，`consciousness_state` 接到 `nt_crystal_core/consciousness.rs`<br>③ `main.rs:52` 换注册源 |
| **验收** | 桌面发一次对话 → `neotrix audit` 落库 → `scripts/truth-surface-baseline.txt` 保持 0 条 |

## 0.5 ⛔ 三处「同名不同型」的**第四次**实例（新发现）

DIR-AUDIT §二已记 3 次（`CapabilityRegistry` 4 份 / `SearchResult` 9 份 / 三个决策引擎）。
本轮发现**第 4 次**，且**性质更严重**：

| # | 实例 | 证据 |
|---|---|---|
| 4 | **嵌套的第三棵记忆树** | `l4_emotion/nt_memory/`（236 文件 / 82,071 行）· `l5_cognition/nt_mind/`（422 文件 / 132,990 行）· `l6_meta/memory/`（7 文件 / 2,048 行，含 `nt_memory_experience_tree.rs`）—— **三处都叫 memory，分属三层** |
| 5 | `nt_jev`（`neotrix/nt_jev/`，14 文件 / 4,312 行）**是真接线的**（`nt_crystal_core/nt_eval_loop.rs:13`、`nt_jev_agentjev.rs:17`、`nt_jev_calibration.rs:18`、`nt_crystal_dialogue.rs:40`、`nt_crystal_task_fusion.rs:41`） | ⇒ **这纠正了 DIR-AUDIT §六「三个决策引擎全未接线」的适用范围**：`nt_jev` + `nt_crystal_core` 是**活路径**（L1 有消费者：`nt_dialogue_tui.rs` `nt_tui_app.rs` `nt_stdin_human.rs` `nt_free_pool.rs` `nt_core_task_dispatcher/nt_dispatcher_core.rs`）。死的只是那 3 个 |

**⇒ 动作**：给「记忆」和「决策」各画一张**唯一裁决表**（哪个是真典、哪些是投影），
挂进 `check-api-surface.sh`，否则下一个 agent 会把活路径当死代码删掉。

---

# 阶段 1：可信状态（3-5 天）——**本轮最高增量**

> 借鉴 `nanobot`（48.6k★，cursor 水位 + 五个留存标签）、`Hindsight`（38k★，delta-ops + move-based 退役）、
> `memU`（14.4k★，**记忆层无 LLM**）、`langgraph`（42k★，两表 checkpoint）。

## 1.1 🔵 记忆记录的**权威头**（loopx reward-memory，Apache-2.0）

| | |
|---|---|
| **支脉节点** | `neotrix-core/src/l5_cognition/nt_mind/nt_mind/experience_tree/mod.rs:29` `ExperienceEntry`<br>`:147` `pub enum Source`（**渠道**语义）· `:108` `Domain`（12 变体）· `:93` `EntryType`<br>5 段协议：`:215` `snapshot` · `:241` `distill` · `:299` `classify` · `:355` `persist` · `:452` `feedback` |
| **外部原文** | *"Every durable record must name `source`, `scope`, **`authority`**, `confidence`, `lifecycle_state`, `supersession`, `revocation`, `expiry`, and `privacy`... **Confidence describes evidence quality; it never increases authority.**"* |
| **动作** | `ExperienceEntry` 加 `authority: Authority` 与 `confidence: Confidence` **两个独立枚举，且携带不变式：`Confidence` 永远不能提升 `Authority`**<br>5 类 authority（对齐 loopx）：`RunBoundEvidence`（*"the overlay itself is not a standing instruction"*）· `HardPolicy` · `SoftPreference`（*"cannot grant publish, merge, write, credential, or production authority"*）· `ProceduralExperience`（*"Retrieval alone has zero patch authority"*）· `WorkingContext` |
| **⚠️ 关键判断** | **不要重载 `Source`**。`Source` 是**渠道**（从哪来），`Authority` 是**权限**（能干什么）。两者正交，重载会破坏 `:147` 的 `Default`。 |
| **验收** | 单测：`Confidence::High` 的 `SoftPreference` 请求写文件 → 拒（**置信度高不等于有权**） |

## 1.2 🔵 五个留存标签（nanobot，**最具体的衰减策略**）

| | |
|---|---|
| **支脉节点** | `experience_tree/mod.rs:355` `persist()` + `:452` `feedback()` |
| **动作** | 采纳 5 个字面量枚举：`Skip`（审计用，不进记忆）· `Correction`（就地替换冲突事实）· `Permanent`（偏好/人格，无视年龄保留）· `Durable`（活动项目上下文，被取代前保留）· `Ephemeral { ttl }`（30 天归档）<br>**标签在落盘前剥离**，不泄漏进召回 |
| **⛔ 反面对照** | Hindsight 的 `memory_units.access_count` 字段**从初版 schema 起就是死的** —— 他们自己的迁移文件写着：*"It is 0 on every row of every install."* 衰减方案**从未上线**。<br>⇒ **不要假设任何「按访问次数衰减」的机制真的存在。** |
| **验收** | 单测：带 `[ephemeral]` 的条目 TTL 到期后不在召回集内，且**内容里不含该标签** |

## 1.3 🔵 delta-ops 取代整体重写（Hindsight `reflect/delta_ops.py`）

| | |
|---|---|
| **支脉节点** | `experience_tree/mod.rs:241` `distill()` + `:355` `persist()` |
| **外部原文** | *"Sections and blocks not mentioned by any op are physically copied through unchanged — there is no LLM-mediated re-emission of unchanged text, so **prose drift is structurally impossible**."*<br>*"**Why blocks are addressed by id and not by index**: An index has to be *counted* by the model, and an off-by-one is still in range, so it silently overwrites an unrelated block and is recorded as a success."* |
| **动作** | `enum DeltaOp { Replace{block_id,text}, Delete{block_id}, InsertAfter{block_id,text} }` + `#[serde(deny_unknown_fields)]`<br>**两层校验**：shape（一个 op 不合 schema → **整条回复拒**）+ reference（未知 `block_id` → **丢该 op，其余照常**）<br>**零 op ⇒ 文档逐字节不变** |
| **验收** | 单测：LLM 返回空 op 列表 → 文件 sha256 不变 |

## 1.4 🟡 move-based 退役 + 无 embedding 归档（Hindsight）

| | |
|---|---|
| **支脉节点** | 记忆存储：`l4_emotion/nt_memory/`（`paged_kv/mod.rs` · `nt_memory_kb/kb_kv.rs` · `vector_index.rs`） |
| **动作** | 两表 `live_facts` / `retired_facts`，**后者无向量列**。`retract()` = `INSERT…SELECT` 然后 `DELETE`<br>⇒ **召回查询不需要 `WHERE status='live'` 谓词**；回滚重算 embedding；换 embedding 模型不会撞维度 |
| **价值** | 召回热路径少一个谓词，且**退役不是软标志**（Hindsight 用 move 而非 flag 是刻意的：*"If a row is in `memory_units` it is live"*） |

## 1.5 🔵 记忆层**无 LLM**，综合交给 agent（memU）

| | |
|---|---|
| **支脉节点** | `nt_mind/`（422 文件）· `neotrix/nt_crystal_core/nt_eval_loop.rs`（已有 distill 流程） |
| **外部原文** | *"`MemoryService` makes no LLM or chat calls; it stores, embeds, and retrieves the skill Markdown the agent prepared."* · *"its **core memory logic is only 500 lines**"* |
| **动作** | `nt_mind` 的 store/embed/retrieve **不做推理**；蒸馏由已有 `distill()` 产出 Markdown 交给它存 |
| **理由** | 记忆服务内部再跑一个 LLM 循环 = **第二个不可审计的 agent 循环**。这是审查盲区。 |
| **⚠️ 对立面** | Hindsight / MetaGPT 都是「记忆服务自己调 LLM」。这是一个**战略分叉**，不是对错 —— 但 **NeoTrix 选 memU 一侧**，因为 R-P79 要求外部技术接进生产可审计的路径。 |

## 1.6 🔵 两表 checkpoint + ULID step id（langgraph）

| | |
|---|---|
| **支脉节点** | `crates/neotrix-neobot/src/nt_store/`（append-only 审计落库）· `nt_cancel.rs:29` `StopToken` |
| **外部原文** | *"**Checkpoints table** — one row per superstep... **Writes table** — one row per node output within a superstep."*<br>`put_writes` 的作用：*"if one or more nodes fail at a given superstep, you can restart your graph from the last successful step **without re-running the sibling nodes that already succeeded**."* |
| **动作** | `run_checkpoints`（每步一行）+ `step_writes`（每节点输出一行）。step id 用 **ULID**（单调 + 可排序 ⇒ 「取最新」是主键范围扫描，不是 `MAX()`） |
| **⛔ 抄这条警告** | langgraph 自己的 `DeltaChannel` 文档：naive prune 会切断链，*"its delta channels would **silently reconstruct as empty (no error raised)**."* ⇒ **任何 resume 路径必须显式发 `Truncated \| Unknown`，不许静默降级。** |
| **验收** | 单测：prune 掉中间 checkpoint 后 resume → 返回 `Truncated`，**不是**空状态 |

## 1.7 🔵 取消：持久化**请求**而非就地取消（deer-flow，83k★）

| | |
|---|---|
| **支脉节点** | `crates/neotrix-neobot/src/nt_cancel.rs:29` `StopToken`（当前是**进程内 bool**）· `nt_daemon.rs:20` `DaemonGate` |
| **外部原文** | *"A non-owning worker now persists the interrupt or rollback request for the live owner, which observes it during lease renewal... **load-balancer routing alone no longer produces a 409**. **The first accepted action wins** even if a retry lands on the owner, and **accepted cancellation competes atomically with owner completion**."* |
| **动作** | `run_control { run_id, action: Cancel\|Rollback, requested_at, requested_by }` 行 + 终态上的 **compare-and-set**。取消延迟 = lease 心跳间隔（**必须写进文档，否则运维以为取消是瞬时的**） |
| **横向对比** | AutoGen 只有协作式 token，**无回滚** · langgraph **完全没有 cancel API** · deer-flow 是唯一做对的。 |

---

# 阶段 2：桌面执行回路（2-3 周）——**真值所在**

> 借鉴 `lahfir/agent-desktop`（1.7k★，**Rust**）、`cua-driver`（26k★ 的 Rust 部分）、
> `OpenAdapt`（1.7k★，VERIFIED 门）、`nanobrowser`（13.8k★）。

## 2.0 ⚠️ 先纠正文档说谎

| | |
|---|---|
| **实测（2026-09-28）** | `neotrix-core/src/l3_embodiment/nt_computer.rs`（477 行）：`trait NtComputer` 有 `screenshot() -> Option<ScreenState>`，**但只有 `LocalComputer` / `MockComputer` 两个实现，文件/进程/sysinfo 抽象**。`LocalComputer::screenshot` 返回 `None`（默认）。<br>`crates/neotrix-neobot/src/nt_computer.rs:91` `trait ComputerBackend` —— **`NoopBackend` 是唯一后端**（`:98`）。<br>`l1_action/nt_io/nt_io_desktop/` —— **只有 2 文件 / 340 行**：`mod.rs`（**8 行**）+ `updater_signing.rs`（332 行，Ed25519） |
| **⛔ 文档在骗人** | `ARCHITECTURE.md:97` 把 `nt_computer/` 宣传为「计算集群」。实测是 filesystem/process trait。**无鼠标 / 键盘 / 截图回路驱动。** |
| **诚实结论** | 这一层是**空的**。2.1-2.4 是真活，不是接线。 |

## 2.1 🟡 元素寻址 + **三态身份判定**（agent-desktop，Apache-2.0）

| | |
|---|---|
| **支脉节点** | 落 `neotrix-core/src/l3_embodiment/nt_computer.rs` 扩 `trait NtComputer`（加 `snapshot` / `act_by_ref`）<br>ABI 参照其 `crates/ffi/include/agent_desktop.h`（2,471 行，`AD_ABI_VERSION_MAJOR 4`，`repr(C)` + size 宏 + bump-or-die） |
| **★ 最值钱的一条** | 三态判定 `IdentityMatch { Match, NoMatch, **Unknown** }` —— *"unavailable evidence is never a false failure."*<br>配套 `LocatorField<T> { Known, Absent, Unknown }`：**证据缺失既不能算匹配，也不能算不匹配。**<br>且**可变控件值（`textfield` 内容）永不作身份**，只有稳定标签作。 |
| **动作** | ① driver ABI 里 `PercentagesSelectorRequest` 与 `CoordinatesSelectorRequest` **并列** —— 分辨率无关性属于**驱动契约**，不是 prompt 技巧<br>② 稳定采样器常量照抄：`>=3` 连续样本 + `>=34ms` 跨度 + `0.5px` 几何容差<br>③ 三种陈旧错误：`STALE_REF` / `AMBIGUOUS_TARGET`（**不任意选一个**）/ `ELEMENT_NOT_FOUND` |
| **验收** | 单测：目标在动作时移位 → 仍可重识别；**可访问性树中途不可用 → `Unknown` 而非 `NoMatch`** |

## 2.2 🟡 `capture_id`：把坐标绑到它被计算的那一帧（cua-driver）

| | |
|---|---|
| **外部原文** | *"With x,y, Driver atomically admits and consumes that exact capture before dispatch; **stale, mismatched, or out-of-bounds captures are refused without fallback.**"* |
| **动作** | 每次截图铸一个 `FrameId`，像素动作**必须携带**它。窗口移动/缩放/重截 ⇒ 坐标作废 |
| **为什么这是全清单唯一真解** | 其余 11 个 GUI 仓（UI-TARS / Agent-S S3 / bytebot / Cradle）**坐标漂移缓解全为 0**。cua-driver 是**唯一**造出了「缺失的指代物」的方案。 |

## 2.3 🟡 `VerificationTier` + 8 态终局（OpenAdapt，MIT）

| | |
|---|---|
| **外部原文（`openadapt_flow/verification.py`）** | 4 级，**数值越小越强**：`INDEPENDENT_SYSTEM(1)` / `INDEPENDENT_SESSION(2)` / `PERSISTED_STATE_REACQUISITION(3)` / `IMMEDIATE_SCREEN(4)`<br>地板 `VERIFIED_EFFECT_TIER = INDEPENDENT_SESSION`<br>反洗白：屏幕回读**不算独立**（*"on-screen read shares the actuation channel"*） |
| **8 态终局（照抄这个词表）** | `VERIFIED` · `HALTED_BEFORE_EFFECT` · **`RECONCILIATION_REQUIRED`**（*"Never blind-retried."*）· `FAILED_PLATFORM` · `CANCELED` · `REJECTED_POLICY` · **`COMPLETED_UNVERIFIED`** · `ROLLED_BACK`<br>核心句：**"Never summarize halt as success."** |
| **动作** | 与 2.4 的 `DeliverySemantics` **合并成一个枚举**（两者中间档同义） |
| **配套** | OpenAdapt 的「claims registry」：每条能力声明必须**指名一个存在的测试文件**，且 `tier` 不得超过该测试的最强证据 —— 这正是 `ARCHITECTURE-MAP-ROADMAP-V2.md` §1-§7 腐化的解药 |

## 2.4 🟡 `DeliverySemantics` → 派生的 `RetryDisposition`（agent-desktop）

| | |
|---|---|
| **外部原文** | 5 态 `Unknown \| NotDelivered \| DeliveryUncertain \| DeliveredUnverified \| DeliveredVerified`，`retry()` 是 `pub const fn` **派生**出来的：<br>`NotDelivered => Safe`；`DeliveryUncertain \| DeliveredUnverified \| DeliveredVerified => **Unsafe**`；`Unknown => Unknown`<br>**反序列化器拒绝不自洽的对**（*"delivery and retry dispositions are inconsistent"*） |
| **动作** | 让「这个动作能不能重试」成为**类型上不可能搞错**的问题，而不是一个调用方用眼睛看的字段 |
| **与 2.3 的关系** | 2.3 的 4 级证据强度 + 2.4 的 5 态投递语义，**合成一个枚举**。这是 neobot 手臂循环的终局类型。 |

## 2.5 🔵 计划级失效门（nanobrowser，15 行）

| | |
|---|---|
| **外部原文** | 执行第 *i* / *N* 个动作前重新快照，比对 branch-path hash 集合；`if !newPathHashes.isSubsetOf(cachedPathHashes)` → **放弃剩余计划**<br>*"Something new appeared after action i / actions.length"* |
| **动作** | 直接落在 neobot 多步计划上。这是**整计划**粒度的失效，不是逐动作重试 |

---

# 阶段 3：成本与上下文（1-2 周）

## 3.1 🔵 成本归因：**唯一插点已存在**

| | |
|---|---|
| **支脉节点（实测复核）** | `neotrix-core/src/l1_action/nt_io/nt_io_provider/anthropic/anthropic.rs:93` **"P0-4 prefix caching: 在稳定前缀边界消息上打 cache_control 断点"**（另有 `:25` `:39` `:43` `:59` `:64` `:209` `:304`）<br>成本账本**已存在**：`crates/neotrix-neobot/src/nt_cost.rs`（142 行，`CostPolicy` `:16` / `price_for` `:71` / `cost_for` `:82`） |
| **缺口** | 那段**「请求时组装、从不写进 transcript」的不可见前缀**是全项目最大成本中心，**零计量**。 |
| **动作** | 在 `:93` 断点处发 canonical block 记录：`zone{input,output}` · `section{static,messages}` · `bucket{system,schema,text,thinking,tool_use,tool_result}` · `tool` / **`skill`** / `role` / `tokens` / `hash=sha1(content)[:8]`<br>派生 `cal_tokens, cached, rewrote, fresh, output, usd` |
| **两条最高杠杆的维度** | ① **skills 是一等维度**（实测 `find skills -type f` = **120** 个文件，无第二家拆这一刀）<br>② **MCP server 是一等维度**（从 `mcp__<server>__<tool>` 还原，配「注入了但从未被调用」的浪费探测器） |
| **诚实契约** | *"总额与账单精确；同一请求内各来源的拆分是近似的。"* 公开残差 `input_err`/`output_err`/`exact: bool`，**不展示假精度** |

## 3.2 🔵 上下文预算**零和算术**（backpass）

| | |
|---|---|
| **支脉节点** | `skills/`（**120 文件**，1.4MB）· `AGENTS.md`（6,087 字节）· `RUST-STANDARDS.md`（28,439 字节）· `neotrix-core/src/skill_registry.rs:14` |
| **外部原文** | 预算 = `bytes(常驻上下文) + Σ bytes(skill.description)`，估算器 `bytes/4`。<br>*"At or over budget, the synthesis prompt goes **zero-sum**: every addition must name the removal or extraction that pays for it."* |
| **动作** | ① 给 `AGENTS.md` / `RUST-STANDARDS.md` / 每个 skill 的 `description` **设字节门**（skill **正文**在被触发前免费）<br>② 抽成 skill 是**泄压阀**（`EXTRACT→SKILL` 免除证据门槛，因为它保留了它移除的每一行） |
| **横向证据** | trendshift 上 `runkids/agents-context-router` 做同一件事（拆 kernel + 路由页 + **强制字节预算**）—— 方向一致 |

## 3.3 🟡 **禁止声明** 学习加权（prime-agent，Apache-2.0）

| | |
|---|---|
| **外部原文** | `harness.py:999-1064` tf-idf 字段加权：`total += idf * (1 + (fields-1) * 0.5)`，idf **按 kind 分别计算**（同类争同一批 top-k 名额）<br>**CJK 感知分词**：中文连续段切成**重叠 bigram**（`修复登录` → `修复`/`复登`/`登录`）<br>`harnessDigestFingerprint()`：**只对渲染器真正打印的字段**做 sha256 + 顺序归一 ⇒ 改 metadata 不会击穿缓存 |
| **动作** | 这三个片段（约 90 行）**几乎可直接移植**。落点：经验树的召回层 |
| **理由** | 你的经验条目是**中英混写**的（见 `ExperienceEntry` 的 `Domain`），纯空格分词对中文召回是坏的 |

---

# 阶段 4：能力诚实度（2-3 周）

## 4.0 🔴 前置：`nt_crystal_core` 是活路径（**纠正既有文档**）

| | |
|---|---|
| **支脉节点** | `neotrix-core/src/neotrix/nt_crystal_core/`（**52 文件 / 21,134 行**）<br>**活消费者（L1，6 处）**：`l1_action/nt_dialogue_tui.rs` · `nt_tui_app.rs` · `nt_stdin_human.rs` · `nt_crystal_llm_bridge.rs` · `nt_free_pool.rs` · `nt_core_task_dispatcher/nt_dispatcher_core.rs`<br>JEV 三原语**已接线**：`nt_eval_loop.rs:13` · `nt_jev_agentjev.rs:17` · `nt_jev_calibration.rs:18` · `nt_crystal_dialogue.rs:40` · `nt_crystal_task_fusion.rs:41`<br>决策三原语文件（`nt_jev/` 4,312 行内）：`primitives.rs` 481 行 · `gate.rs` 480 行 · `eval.rs` 569 行 |
| **⛔ 纠正** | DIR-AUDIT §六「三个决策引擎全未接线」**仅对那 3 个成立**。`nt_jev` + `nt_crystal_core` 是**生产活路径**。<br>⇒ **禁止把 `neotrix/` 目录当死代码清理。** 上一轮若照 DIR-AUDIT 读，会误删 52 个文件。 |
| **动作** | 0.4 脱 stub 的 `UnifiedApiImpl` **应委派到这里**（不是新建路径） |

## 4.1 🟡 负面证据**分类**（backpass）—— 见 §0.3

| | |
|---|---|
| **支脉节点** | `experience_tree/mod.rs:452` `feedback()` |
| **动作** | `enum NegativeClass { Harm, NonCompliance, Irrelevant }` 落在每条记忆证据上<br>路由：`Harm → 删/弱化`；`NonCompliance → 加强`；`Irrelevant → 丢弃`<br>删规则需 **≥2 个独立会话的 `Harm` 类**证据 |
| **配套（背书）** | **逐字引文 + 程序化包含校验**（*"Every claim must carry a verbatim quote... Quoteless items are discarded"*）≈ 20 行，是防「自进化循环编造自己的教训」的最便宜防御 |

## 4.2 🟡 经验值的 review-due + 三级注意力（oh-my-hermes）

| | |
|---|---|
| **支脉节点** | `experience_tree/mod.rs:29` `ExperienceEntry` |
| **动作** | 加 `provenance: Provenance`（MaterialBacked / Uncovered / ModelAdded / ExternallyVerified）+ `review_due: Date` + `attention: Active\|Reference\|Archive`<br>**`Uncovered` 不能携带断言** |
| **理由** | `Source`（渠道）与 `Provenance`（证据等级）**正交**，别焊死 |

## 4.3 🔵 记忆的 **GC 生命周期**（hermes Curator，MIT）—— 全清单唯一有的东西

| | |
|---|---|
| **支脉节点** | `experience_tree/mod.rs:29` + `l4_emotion/nt_memory/` |
| **外部机制** | 状态机 `active → stale(14d) → archived(30d) → purge(TTL)`<br>每次变更写 **content-addressed ledger**：blob 存一次，100 条同文件变更只占 1 个 blob<br>`rollback(entry_id)` **精确恢复该条触碰的文件**<br>**先快照后变更**，**捕获失败则不改**（fail-closed） |
| **为什么必须有** | **prime-agent / loopx / backpass 三家都会无限累积死条目** —— 它们的条目只会 `version+1`，永不退役。这是所有自进化循环**缺的另一半**。 |

## 4.4 🟡 「模型可见 ⇒ 必须已记日志」（deepseek-harness，238k★）

| | |
|---|---|
| **外部原文** | *"A runtime invariant checks model requests are **reconstructable from the log**. New model-visible inputs require session events."* |
| **支脉节点** | `l0_substrate/nt_core_telemetry.rs`（**1,572 行**，10+ 事件变体）· `crates/neotrix-neobot/src/nt_audit.rs`（`AuditEvent` `:27`）· `nt_store/` |
| **动作** | 事件分**三域**：durable log 事件 / live `agent/*` 事件 / capability-seam 事件。加**运行时不变量**：任何到达模型的输入必须能从 append-only log 重建 |
| **价值** | 这是 C4/vision 边界**可机器检查**的形式 |

---

# 阶段 5：策略地板（3-5 天，可全程并行）

## 5.1 🟢 非不可宽化策略地板（neobot，**已具备大半**）

| | |
|---|---|
| **支脉节点（实测）** | `crates/neotrix-neobot/src/nt_policy.rs:75` `evaluate_policy`（fail-closed）<br>deny 全集：`:77` `human-control` · `:98` `:106` `workspace-jail` · `:129` `unknown-tool` · `:131` `default-deny`<br>`:137` `evaluate_extra_deny`（坏规则**仍拒**）· `nt_audit.rs`（先写 audit → 再执行） |
| **缺口** | 已是 default-deny + 拒绝优先 + 畸形规则拒绝 ✅，**但没有任何机制阻止一个新 bug 或一次新加的 override 把地板降低**。 |
| **动作** | 把「被允许绕过批准阶段的能力集合」变成**测试钉死的冻结数据**。任何新增 bypass 必须同时改冻结集 → 改动必然经过 review |
| **验收** | 一条单测：冻结集之外的任何 capability 请求批准 → 拒 |

## 5.2 🔵 DNS qtype 白名单（承 2026-09-27 路线图 1.1，**仍未做**）

| | |
|---|---|
| **支脉节点** | `l1_action/nt_io/nt_io_provider/common/egress_types.rs:14-21` `SandboxEgressRule { host, port, allow }`<br>**全文件零 DNS 概念** —— 无 qtype、无 resolver 管控、无 label 长度上限 |
| **动作** | 加 `qtype` + `label_len_max` / `name_len_max`；`TXT`/`NS`/`SRV`/`CAA`/`DNSKEY` 一律拒；把 `dns_allow` 从「学习提示」改成**过滤器** |
| **为何仍重要** | 深挖 7 个沙箱仓：**没有一个做 qtype 过滤**。这是事故已真实发生的那一类。 |

## 5.3 🔵 「尝试」与「结果」是两个独立字段（OpenAI DNS 事故教训）

| | |
|---|---|
| **支脉节点** | `l0_substrate/nt_core_telemetry.rs`（事件 schema 落点）· `l6_meta/nt_safety_monitor.rs` |
| **动作** | 事件加 `attempt: Attempt` 与 `outcome: Outcome` **两个独立字段**<br>**硬规则：任何情况下不得让 outcome 衰减 attempt 的严重性。** 返回 NXDOMAIN 的越权查询**仍然是越权查询**。 |
| **配套** | CI 断言 A：*检测器覆盖范围 ⊇ 执行范围*（事故中基础设施异常 DNS 检测器**把受影响环境排除在外**） |

---

# 执行状态（2026-09-28 实测，非文档转述）

| 项 | 2026-09-27 路线图 | **2026-09-28 实测** |
|---|---|---|
| 0.1 脱 `UnifiedApi` stub | 🟡 待做 | ⛔ **仍是 stub**（`stub.rs:275` 单元结构体，`:289` 返回字面量，`main.rs:387` `:407` 在用） |
| 0.2 注册表收敛 | ⛔ 待做 | ⛔ **未收敛**：`CapabilityRegistry` **4** 份 · `ToolRegistry` **3** 份 · `SkillRegistry` **4** 份 |
| 0.4 `maturity_audit` 接 CI | 🟢 已有资产 | ✅ **已进 CI**（`ci.yml:214` `capability-truth`，阻塞态）+ `Epistemic` 已接线并有 `tests/epistemic_wiring.rs` 验证真实路径 |
| 0.3 四 orchestrator 择一 | ⛔ 待做 | ⛔ **未做**：`OrchestratorConfig`/`OrchestratorStats`/`Orchestrator` 等 **20 处**定义 |
| 分层依赖门 | 🟡 无 | ✅ **已建** `scripts/check-layer-deps.sh` + 92 行基线；实测 **84 sites / 92 baseline，已解 8**，`--strict` **PASS 0 new** |
| 5.1 GUI 驱动 | 🟡 承认是空的 | ⛔ **确认是空的**：`NoopBackend` 唯一后端；`nt_io_desktop/` 2 文件 340 行 |
| 2.1 GUI 元素寻址 | 🔵 决策面 | ⛔ 未做（新增 2.1-2.5 五项，见阶段 2） |

**⇒ 判断：2026-09-27 路线图的 0.1 / 0.2 / 0.3 三项一周内未动。**
本轮**不重排优先级**，而是把**已经存在的资产**（`capability-truth` 门、`layer-deps` 门、`Epistemic` 接线、`nt_crystal_core` 活路径）标出来 —— 路线图最大的风险不是顺序错，是**读文档的人以为已经做了**。

---

# 依赖图

```
0.1 证伪门(0.2) ──→ 阶段 1 全部记忆改动 ──→ 4.x 自进化
0.1 注册表收敛 ────→ 0.5 唯一裁决表 ──→ 防误删活路径(nt_crystal_core)
0.3 maturity CI ───→ 能力诚实度可验证
0.4 脱 stub ───────┬─→ 3.1 成本归因(先有真实流量)
                   └─→ 4.0 委派到 nt_crystal_core(已存在的活路径)
1.1+1.2 记忆权威/留存 ──→ 1.3 delta-ops ──→ 1.4 move-based 退役
1.6 两表 checkpoint ──→ 1.7 取消 CAS
2.1 元素寻址 ──→ 2.2 capture_id ──→ 2.3+2.4 合并终局枚举 ──→ 2.5 计划失效门
3.1 成本归因 ──→ 成本感知路由
5.x 全独立，可全程并行
```

**强依赖只有 5 条**，其余可并行。

---

# ⛔ 许可红线（本轮实测，逐条）

| 仓 | 许可 | 判定 |
|---|---|---|
| `tdeverx/contained-app` | **PolyForm Noncommercial 1.0.0** | ⛔ **不可用**。且实测**无 XPC/MCP/URL scheme** —— 零 agent 面 |
| `multica-ai/multica` | 自定义（禁托管服务） | ⛔ **不可抄码**，只读想法 |
| `volcengine/OpenViking` | **AGPL-3.0** | ⛔ 不可抄。只读 L0/L1/L2 渐进披露设计 |
| `vectorize-io/agent-memory-benchmark` | **无 LICENSE 文件** | ⛔ 不可依赖。**vendor 那 1 页 harness 代码**，数据集需署名 |
| `digipulse-engineering/GAAI-framework` | **ELv2**（非 OSI） | ⛔ 只读设计（双轨进程隔离） |
| `lopopolo/harness-engineering` | **CC-BY-4.0** | ⚠️ **散文需署名**。只引协议结构，不抄进源码 |
| `bytedance/deer-flow` / `PrimeIntellect-ai/prime-agent` | MIT | ✅ 可用（但 prime-agent LICENSE 头写 `Copyright (c) 2025 Mario Zechner`，vendor 前查清归属） |

---

# ⛔ 引用的数字陷阱（防止下轮照抄）

| 陷阱 | 实测真相 |
|---|---|
| **mem0 的 92.5** | README 自述：*"Scores reflect Mem0's **managed platform, which includes proprietary optimizations not available in the open-source SDK**"*。规模自降 LoCoMo 92.5 → BEAM(10M) **48.6**。⇒ **不可作为 OSS 可达数字引用** |
| **jemv 决策面效果** | jev-drone 头条数字是 **n=1**；更早 3-seed 配对比较**无优势（0/3）** |
| **Hindsight 的 `access_count`** | 迁移文件自承**从初版起就是死的，每行都是 0** ⇒ 衰减方案从未上线 |
| **ROUPE 的 73k★** | README 主打改名 + 付费层 + 工具数。**星数不是机制证据** |
| **trendshift yearly 的 `gained`** | ≈ 总星数（20/25 仓 r≈1.0）⇒ **不是周期增长，比例无意义**。只有 monthly 板有相对增长信息，且只有 `r ∈ [0.83, 0.99]` 段可信 |
| **Jev/Laya 星群** | 17 个仓跨 6 榜同品牌播种。**14/17 是 astroturf** ⇒ 取架构，**忽略生态** |

---

# 三条最该抄的（如果只做三件）

1. **0.2 证伪门** —— 全清单 11 个自进化仓**无一**证明自己有效。这是唯一能让 NeoTrix 领先而非跟随的东西。
2. **2.2 `capture_id`** —— 把坐标绑到它被计算的那一帧，陈旧即拒**且不回落**。其余 11 个 GUI 仓坐标漂移缓解**全为 0**。
3. **0.5 唯一裁决表 + 4.0 活路径纠正** —— 本轮差点让下一轮把 52 个文件（`nt_crystal_core`，**L1 有 6 个活消费者**）当死代码删掉。「导出 ≠ 调用」这条已经错过 3 次（`CapabilityRegistry` 4 / `SearchResult` 9 / 三个决策引擎），**第 4 次是 JEV —— 只不过这次是反过来的错：它是活的**。
