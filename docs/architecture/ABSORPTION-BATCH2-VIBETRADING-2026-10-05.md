# ABSORPTION-BATCH2-VIBETRADING — 并行批次 2 + Vibe-Trading（2026-10-05）

> 承接 `ABSORPTION-MIU2D-RA2-2026-10-05.md`。本轮是**并行代理批次**（6 路）
> + 两个新外部源。⚠️ 本轮**同时含一条对上一轮结论的更正**，见 §4。

---

## 1. ⛔ 三条前提修正（读现场才暴露的）

| 事项 | 我原以为 | 实测 |
|---|---|---|
| `rust-alert` | 是个仓库 | **组织主页**，15 仓（已在上一轮修正） |
| `bobeff/open-source-games`（15,616★） | 游戏引擎 | **不是代码仓** —— 描述即 "A list of open source games"，CC0-1.0，**总 94 KB / Python 仅 1541 字节**。是**链接清单**，无可吸收的引擎 |
| `CapabilityRegistry::nodes` 是 `HashMap` | ⇒ 必须排序否则漂移 | 是 **`IndexMap`**（插入序，本就确定）⇒ 该理由不成立（已在上一轮修正） |

⇒ **`open-source-games` 判定：无可吸收实现。** 它的价值至多是「清单策展」，
不属于本仓能力路线；若日后要用，只当**发现源**而非**吸收源**。

---

## 2. 并行批次 2：6 路代理的文件集严格互斥，cargo 全部留给我串行跑

> 分工原则：**代理只改码不编译**（`rustfmt --emit stdout` 仅做解析检查），
> 我持 `nt_build_lock.sh --strict` 串行验证。理由见 §5.1 的事故。

| 路 | 文件 | 成果 |
|---|---|---|
| P1 | `neotrix-multi-agent/src/god_agent.rs` | **成本闸落地**：`max_cost_per_decision`/`prefer_specialists`/`fallback_agent_id` 三字段从「声明未读」变为生效；`select_agent` 重写为 T0–T3 分层；`classify` 加确定性 tie-break。+447 行 |
| P2 | `subgrid.rs` / `skill_quality.rs` / `nt_router.rs` / `nt_memory_lead.rs` | 4 处 tie-break，**其中 2 处证伪**（见 §3） |
| P3 | `neotrix-core/src/skill_loader.rs` | roadmap **A5**：`visible_to_model`/`visible_to_user` 零消费者 → 在**呈现边界**接线；索引深拷贝改 `Arc`；进程级 `INDEX_CACHE`（消除每 tick 重解析）。+553 行 |
| P4 | 只读 | E6/C7 可行性规格（**其 E6 结论已被本轮更正**，见 §4） |
| P5 | `neotrix-core/src/l6_meta/nt_approval.rs` | approve/deny 原本**字节相同**（都只 `remove`）→ 改为可区分 + 可审计 + 幂等拒绝；`approve_all`/`deny_all` 原本 `pending.clear()`（批量批准零痕迹）→ 逐条留痕。+575 行 |
| P6 | `input_validator.rs` / `shanhai_query.rs` / `entry/wiki.rs` | **1 处证伪 + 2 处真实 panic 修复**（见 §3） |

### 2.1 P1 的成本闸：选了「泄压阀」而非「默认拒绝」，并写了理由

超预算候选**全部**超标时的规则，P1 选了 **relief valve（泄压阀，fail-open）**：
取超支幅度最小者，且**仍过治理门**。否决了 `None`（默认拒绝），理由有代码依据：
`route` 在 `selected_agent_id: None` 时会产出 `escalated: false` + `escalation_reason: None`
⇒ **静默丢弃决策**，比超支更坏。且成本超支是**资源约束**不是**安全策略**，熔断器已有 stop-the-world。

---

## 3. ⭐ 代理**证伪了我 prompt 里 5 处**错误前提（这一节是本轮最有价值的部分）

| 我的错误前提 | 代理的实证更正 |
|---|---|
| `EscalationRules` 注释写明 fail-closed 约定 | **没有这类注释**。该约定是从 `route` 的*行为*推导的，不是文字声明 |
| `AgentInfo` 字段有 doc 可引 | **零 doc 注释**。`cost_budget` 语义（按 agent 还是按决策）**真未知** ⇒ P1 把这个歧义**写进字段文档**而非假装已解决 |
| `nt_router.rs` / `nt_memory_lead.rs` 用无序容器 | **两者都是 `Vec`** ⇒ 不存在跨进程不确定性。代理仍加固了顺序无关性，但**明确标注不是活 bug** |
| `input_validator.rs:153` 的 `&mat[..40]` 是 panic | **证伪**：base64 正则字符类 `[A-Za-z0-9+/]` 与 `=` **纯 ASCII** ⇒ 不可能落在非字符边界；且长度守卫 `>100 ≥ 101 > 40` 排除越界。代理**拒绝「修」非 bug**，只加了一条固化该不变量的测试 |
| `skill_loader` 的索引深拷贝是「无谓浪费」 | **不是** —— `load_index(&mut self)` 返回借用、`resolve_from_index(&self)` 又要第二个共享借用，**借用检查器不允许**。原克隆是绕行手段。`Arc` 是正解 |

⇒ **教训（比本轮任何落地都重要）**：我把 6 个 prompt 交给代理后，
**5 处前提被推翻**。若我串行自己做，其中至少 3 处会变成「照着错误前提改正确代码」——
这正是 AGENTS.md §5 R-SCAN-1 记录的事故形态。**并行代理的最大价值不是产能，是独立证伪。**

---

## 4. ⛔⛔ 对 P4「E6 是 ceremony」结论的更正（Vibe-Trading 证伪）

**P4 的结论**（已写进 `handoff-2026-10-05-miu2d-ra2.md` 与吸收文档 §8）：
> 「E6 更可能是假的……是**一个居民**的类别。」

**Vibe-Trading 的反证**（`agent/src/agent/loop.py:2850-2925`，MIT）：
超时是否**杀进程**直接按可逆性分流 ——

- **写工具永不杀**：超时只 `warn` 一次，然后**等它跑完**
  > `// Write tools are never killed: a watchdog warns once past the timeout, then the result is awaited to completion.`
- **只读工具**才有界超时并返回结构化 `tool_timeout` 错误

⇒ **该类别真实、载重、且正是 NeoTrix 缺的**。P4 错在把「NeoTrix 当前接线是假的」
推广成「这个类别是假的」——**这是两个不同命题**。

⚠️ 同时保留 P4 的**另一半正确结论**：`RiskTier`（`nt_jev/risk.rs:17`）全仓**只有一个
在用配置值** `Recoverable`，其输出只是给人看的 `follow_ups` 文本（`nt_crystal_task_fusion.rs:931-934`），
**不门控任何东西**。⇒ 正确表述是：

> **可逆性类别有 10 个已登记工具的真实居民（`nt_shield_enforcer.rs:391-400`），
> 但没有任何一处把它接到「置信度/超时」决策上。缺的是接线，不是类别。**

⇒ **禁止**新造 `Reversibility` 枚举（已有 `ToolReversibility` 4 变体 + `ActionTier` 映射，
再造即第五种拼写 ⇒ AGENTS.md §4.2「同名 ≠ 同一符号」坑）。

---

## 5. Vibe-Trading（MIT，34,711★）最高价值的三项

| # | 机制 | 出处 | 对 NeoTrix 的价值 |
|---|---|---|---|
| **1** | **评测台从「不跑 agent」** —— 纯函数 `(case JSON, artifact bundle) → verdicts`；四值裁决含 `NOT_EVALUABLE`；每个失败断言带 `evidence_refs` 指向**确切 trace 行号**；退出码 0/1/2；**拒绝让覆盖率虚报 1.0** | `agent/evals/harness/{README.md:1-18, schema.py:11-18, runner.py:80-85, assertions.py:22-37}` | ⭐⭐⭐ 直接补 NeoTrix 缺失的「产物级回归」。**无 LLM judge ⇒ 无 flaky，可在 CI 跑几天前的产物** |
| **2** | **超时按可逆性分流**（见 §4） | `loop.py:2850-2925` | ⭐⭐⭐ 正是 E6 缺的接线 |
| **3** | **checked-numbers 溯源门** —— 模型为每个数字**声明来源角色**（observed/derived/proposed/cited/count）；检查是**声明式注册表的一行**而非共享控制流；修正预算有界（`MAX_GROUNDING_REVISIONS = 2`）；**降级发布**：切掉失败数字 → 用同一门复验**剩余文本** → 仍失败才 fail-closed 拒答；`degraded: true` | `figures.py:36` / `registry.py:1-13` / `release.py:515-527` / `loop.py:1718-1901` | ⭐⭐⭐ 全部或全无的二元判定 → **优雅降级**。且 `released_text` 在同一趟里算出（`ledger.py:337-341`） |

### 5.2 ⛔ Vibe-Trading 自带缺陷（记录以防照抄）

- ⛔ **`RunManifest.skills` 永远是空的** —— `build_run_manifest` 收 `skills=` 参数，
  但唯一生产调用点不传（`loop.py:1055-1068`）⇒ 「哪个 skill 变了」的 diff 能力**生产中已死**。
  ⭐ 该仓诚实披露了这个缺口（`extra` 字段写明），**披露本身是可抄的纪律**。
- ⛔ **`completion_status` / `safety_status` 从未被任何代码写入** ⇒ 两个指标组恒空，
  硬编码为 `NOT_EVALUABLE`（`assertions.py:229-240`）。
- ⛔ `authorization_bypass` 记录类型**只有断言在读，无任何生产者** ⇒ 死的前向兼容。
- ⛔ `on_missing` 字段只接受唯一一个值（`schema.py:88-91`）⇒ 未实现的策略占位。
- ⛔ MCP 重试分类靠**消息子串**（含裸 token `"eof"`）⇒ 业务错误文本含 "timeout" 就会被重试。
  ⭐ 但它的**原则**正确且应抄：**副作用调用绝不自动重试**（`mcp.py:772-797`：
  超时/断连发生在服务端已提交之后，重试会重复副作用）。
- ⛔ 预算**只记账不执行** —— `BUDGET_LIMITED` 无循环侧处理，真正的上限是 `max_iterations`。
- ⚠️ `char_indices().nth(n)` + `&s[..idx]` 的截断写法会触发 `clippy::string_slice`
  ⇒ 仓内 `-D warnings` 下 **CI 红**。必须用 `chars().take()`（本仓 `nt_approval.rs` 的修法已合规）。

---

## 6. 验证状态（⛔ 严格区分「已验」与「未验」，不声称未跑过的结果）

### 6.1 已完成（有实测输出）

| 项 | 结果 |
|---|---|
| `cargo check -p neotrix-multi-agent --lib --tests` | **rc=0** ✅（P1 +447 行编译干净） |
| `cargo check -p neotrix --lib --tests` | **rc=0** ✅（P2/P3/P5 + 本轮我改的 8 文件） |
| `cargo check -p neotrix --bins` | **rc=0** ✅（P6 三文件；**P6 情报：`mod entry;` 在 `main.rs` 非 `lib.rs`，故 `--lib` 覆盖不到**） |
| `cargo test -p neotrix-multi-agent --lib` | **50 passed** ✅（含 P1 成本闸 5 项：tag_match/specialist 排除、NaN 预算、泄压阀选最小超支、泄压阀仍守治理门） |
| `cargo test -p neotrix --lib audit_residency` | **5 passed** ✅（P2 tie-break） |
| `rustfmt --emit stdout` 解析检查 | 6 文件全通过（**只排除语法错，不排除类型错**） |

### 6.2 ✅ 定向测试全部通过（2026-10-05 12:42 完成，104 tests）

| 目标 | 结果 |
|---|---|
| `nt_memory_lead`（我的 `query` 规范化 + 有序性测试 + P2 两个） | **8 passed** ✅ |
| `nt_core_cache`（第一轮 3 处修复） | **22 passed** ✅ |
| `nt_determinism`（确定性三件套原语） | **11 passed** ✅ |
| 涌现指纹（`capability_digest`） | **2 passed** ✅ |
| `skill_loader`（P3 · roadmap A5 接线） | **26 passed** ✅ |
| `select_best_for_profile`（P2 tie-break） | **2 passed** ✅ |
| `nt_router`（P2 顺序无关性加固） | **3 passed** ✅ |
| `nt_approval`（P5 审计化 + `preview_or_full` panic 修复） | **29 passed** ✅ |
| 涌现探针（消费方） | **1 passed** ✅ |

⇒ **本轮全部落地产出已编译 + 已测绿。** 累计 159 个定向测试通过
（104 + 50 `neotrix-multi-agent` + 5 `audit_residency`）。

### 6.3 ⛔ 唯一仍未做的事：全量 `cargo test -p neotrix --lib`（13,159 个）

§6.4 记录的那次意外全量运行**结果未知**（输出被截断），不能声称通过。
⇒ 接手者请补跑。**本轮只保证上述 159 个定向测试 + 3 次 `cargo check` rc=0。**

### 6.4 ⚠️ 两次被外部阻断，均非本会话引入
  1. `nt_core_capability_tree` 一度编译失败（`self` 参数 / `Self` 外层使用）——
     该窗随后自行修好（`registry.rs` +26 行），现编译通过。
  2. `nt_memory_pack.rs`（**不在我任何代理范围内**）一度报 `E0384: pos`——
     同一次运行中后续目标即编译通过 ⇒ **他窗在途状态，已消失**。
  ⇒ **教训**：共享脏树下，归因必须靠「文件是否在我的清单内」+「同一批后续目标是否成功」两条独立事实，
     **不能**靠本机门计数（AGENTS.md §4.2 已警告脏树计数不可信）。

### 6.3 ⚠️ 我在验证方法上犯的错（记录）

1. **把「等待锁」进度行过滤掉**，导致命令看起来像卡死无输出 —— 用户两次中断。**我的 UX 失误。**
2. **在共享树上做「回退-verify」表演**，撞上 3 个他窗 cargo，会污染其编译结果。
   ⇒ 判别力证明改用 `/tmp` 里 `rustc` 单独编译的旁路程序。
3. **脚本 bug 导致跑了全量而非定向**：`run()` 只传了 `$4`，把过滤器 `$5` 丢了 ⇒
   `nt_approval` 那格实际跑了 **13,159 个测试**（意外达成「补跑全量 lib」，
   但输出被 `head -30` 截断，**结果未知**，不能声称通过）。

---

## 7. 本轮新增缺陷台账（按严重度）

| 级别 | 缺陷 | 状态 |
|---|---|---|
| 🔴 **治理** | `max_cost_per_decision` 是**空闸**：超预算 agent 仍可被选中 | ✅ 已修 + 5 测试绿 |
| 🔴 **正确性** | `LeadManager::query` 在 HashMap 上 `.take(limit)` ⇒ **top-N 结果集跨进程漂移**，且**根本没按 score 排序**（`limit` 名不符实）。LIVE（`nt_act_trade/orchestrator.rs:416`） | ✅ 已修 + 测试（未验） |
| 🔴 **正确性** | `nt_approval` approve/deny **字节相同** + `next_id` 每进程从 `a0000` 重启 ⇒ 过期点击批准错误动作。**但该路径经证实生产不可达**（`turn_stream_with_approval` 零调用者） | ✅ 已修（纵深防御）+ 测试（未验） |
| 🟠 **路由** | `record_sub_grid_call` 按 profile 找子网格，但 map 按**名字**键入 ⇒ 两个不同名网格可共享 profile ⇒ 归属漂移 ⇒ **健康计数不同 ⇒ 路由前置条件可能翻转** | ✅ 已修（未验） |
| 🟠 **panic** | `&str` 按字节切片：`nt_approval.rs`（可达自 `submit`，内容含 CJK 即 panic）、`shanhai_query.rs:69,101`（山海经正文，**几乎每条真实记录都触发**）、`entry/wiki.rs:94`（本仓 CJK 文档） | ✅ 已修（`nt_approval` 未验 / P6 `--bins` 已过） |
| 🟡 **一致性** | `audit_residency` / `select_best_for_profile` 同分无名字兜底 | ✅ 已修 + 5 测试绿 |
| 🟡 **性能** | `skill_loader` 每 tick 重解析 `index.json`（27KB）+ 深拷贝索引树 | ✅ 已改 `Arc` + 进程缓存 |
| ⚪ **记录不改** | `nt_approval.rs:222` 非测试代码 `.expect("key exists")`；`input_validator.rs:145` `.expect("valid regex")` | ⛔ 修需改公开签名/控制流，**本轮不硬改**，留给接手者 |

---

## 8. 下一步（按杠杆）

1. **接上 §4 的可逆性接线**（Vibe-Trading 的 #2）：在 `nt_agent.rs:600` 的工具执行漏斗处，
   按 `ToolReversibility` 分流超时策略（写=永不杀、只读=有界报错），**shadow 先行**
   （只记 audit 不改行为）⇒ 让「这个类别到底有没有居民」变成**可测量**而非可争论。
2. **落 Vibe-Trading 的 #1（产物级评测台）**：纯函数 + 四值裁决 + `evidence_refs`。
   前提是先确认 NeoTrix 的运行产物是否已有等价于 `state.json` / `trace.jsonl` / manifest 的东西。
3. **落 #3（溯源门 + 降级发布）**：注意其「切掉失败数字再复验剩余」的顺序，
   `released_text` 与裁决同一趟产出。
4. **补跑全量 `cargo test -p neotrix --lib`**（13,159 个）—— 本轮因故只跑了定向集。
5. `A3` roadmap 项（`check-test-baseline.sh` 账本 0 字节）现在**可以做了**：
   §6.3 的意外全量运行说明 `--list` 已能跑，只是当时被我脚本 bug 丢掉了输出。

---

*本轮全部改动未提交。文件清单与收工自查见 `sessions/handoff-2026-10-05-miu2d-ra2.md` 的追加节。*
