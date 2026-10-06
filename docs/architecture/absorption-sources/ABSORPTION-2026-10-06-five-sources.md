# 五源吸收判定（2026-10-06）

用户提交 5 个源。**先过吸收前置门（LICENSE），再谈机制** ——
未过门者只取设计，不取码（依据 `LICENSES.md` 的分档口径）。

| 源 | License | 判定 | 取什么 |
|---|---|---|---|
| `harry0703/MangoDisk` | **GPL-3.0** | ⛔ **NOT-CODE** | 仅设计：treemap 式空间分析、安全优先的清理器 |
| `libingzheren/Jev-Mem` | MIT | ✅ 设计（Python 跨语言，未接线） | 见下「Jev-Mem」 |
| `tinyhumansai/openhuman` | **GPL-3.0** | ⛔ **NOT-CODE** | 仅设计：local-first agent harness |
| `uber/ADR` | **Apache-2.0** | ✅ **可取码 + 取设计** | 见下「uber/ADR」⭐ 本轮主要来源 |
| `codewhale-hq/Codewhale` | MIT | ✅ 设计（已于 2026-10-05 吸收过授权栈） | 补入台账 |

台账：`repos.csv` 现 **525 条**（本轮补 3 条新行 + 2 行补 `boards` 裁决）。
⚠️ 第 498 行 `trendshift.io` 是**站点非仓库**（第 6 列另有用途），字段数与表头不符
是**既有形态**，本轮只记录、不擅改。

---

## ⭐ uber/ADR —— 本轮主要来源（Apache-2.0，已部署 Uber 生产，MLSys 2026）

`arXiv:2605.17380`。五组件：Discovery / Observability / **Benchmark** / **Detection** /
Prevention（**Prevention 未开源** ⇒ 「阻止危险动作」那一层我们**拿不到**，只能自建）。

### 已吸收并落地：审计**记摘要不记内容**

**源**：其 `run_manifest` 明确 *「Paths, directory names, host identifiers,
environment values, prompts, and file contents are not stored」* ——
要证明「审批的就是这份内容」只需 SHA-256 摘要。

**我方实测到的缺陷（本轮已修）**：
`l6_meta/nt_approval.rs::describe_action` 曾把 `content_preview` / `diff`
的**原文**（≤60 字符）拼进 `description`，而 `description` 被
`ApprovalAuditEntry::description` **快照进审计轨迹**。
⇒ **每写一次文件，就把它开头 60 个字符留在长期留存的审计里**；
写 `.env` / `credentials` 时，那 60 个字符**就是密钥本身**。

**落地**：
· 内容/diff 一律改为 `content_fingerprint` = `<N> chars, sha256:<12hex>`
  —— 路径与规模保留（审批人仍能定位与复核），**内容不再被抄走**。
· 命令/工具参数加 `redact_assigned_secrets`：遮蔽 `KEY=value` 与 `--flag VALUE`
  **两种形态**的密钥**值**，**保留键名与被遮值长度**。
  命令**本体保留** —— 遮掉命令等于让审批失效。
· 金丝雀测试 6 条（`audit_content_leak_tests`）。

**⚠️ 遮蔽器的两次实测教训（都写进注释了）**：
1. 首版用 `key.contains("key")` ⇒ **`KEYBOARD=1` 被误遮**。
   而误遮的代价 = **命令不可读 = 审批失效**，比漏遮更难发现。
   ⇒ 改为**按 `_`/`-`/`.`/数字切段后整段命中**才判定。
2. 首版只处理 `KEY=value` ⇒ 金丝雀立刻抓到 `--api-key sk_live_...`
   —— **CLI 里 `--flag VALUE` 比 `KEY=value` 更常见**。

### 已吸收并落地：**结构化路由契约 + 失败朝 escalate**

**源**：`decision_contract: "structured_risk_route_v1"`，显式 `BENIGN_EXIT` / `ESCALATE`；
且 *「Missing, malformed, or internally inconsistent structured output
**fails toward `ESCALATE`**」*。

⇒ 与本仓既有判据同源（`Authorization`/审批链的「静默降级只朝严格方向」）：
**解析失败必须 escalate，不能回落成放行**。本轮 `--sandbox` /
`--approval-mode` 的 `from_str` 改 `Result` 正是这条纪律的落地
（`2169b633`）。uber/ADR 提供了第三方实证与措辞。

### 明确**未**吸收（记录理由，避免下一个人重复评估）

| ADR 的做法 | 为何不取 |
|---|---|
| **双 agent 检测**（Tier-1 `gpt-4o` 分诊 + Tier-2 `claude-sonnet-4-6` 深推理） | 依赖外部 LLM 与 API key。本仓是**无 I/O 的纯规则实现**，引入即改变部署形态。**它的架构思想**（便宜高召回分诊在前、昂贵推理只对可疑者）已被本仓多处采用，但**不引入其模型依赖** |
| **Discovery**（清点端点上的 AI 工具/MCP server 并标记未知面） | 本仓已有等价物（`nt_capability_registry` + `agent_guardrails`），重复实现无收益 |
| **Prevention 组件** | **未开源**，无从吸收 |
| **ADR-Bench 304 任务 / 134 MCP server** | 是**评测工件**，非运行时机制；引入需容器化隔离（其 README 明令 *"must be run in an isolated environment"* 且依赖带已知 CVE）⇒ 不进生产树 |
| **供应链冒充测试样本**（`location_harvester` 注册成 community 身份） | ⭐ 思路值得记：同一恶意工具可用**「合法身份」**注册。⇒ 本仓若将来做 MCP 白名单，必须校验 `type` 与**实际来源**一致，不能只看 `type` 字段 |

### 它的实测数字（作为我们目标的参照，非承诺）
ADR-Bench 上 precision **1.000** / recall **0.667** / **0 误报**；
基线 LlamaFirewall precision 0.167、40 误报。
⭐ **注意**：ADR 自己选择 recall 0.667 而保 0 误报 ——
**在「审批/阻断」这类场景里，误报的代价高于漏报**。
这与我们「Deny 不可被 Ask 覆盖」「静默降级只朝严格方向」是同一条取舍。

---

## Jev-Mem（MIT，Python 跨语言）

`System-One Controlled Agentic Memory`。**本轮只取设计，未接线** ——
理由：本仓记忆栈（`nt_memory_kb` / `tiered_memory` / `paged_kv`）已挂载且有 2663 行
在编译树内，此时引入第二套记忆范式的**风险高于收益**。
⚠️ 记入台账是为了**下次有人提及时不必从零调研**。

## openhuman / MangoDisk（均 GPL-3.0 ⇒ ⛔ 不取码）

只记两点设计观察：
· **openhuman**：local-first + second-brain + MCP 的**组合方式**值得参考，
  但 GPL-3.0 与本仓分发口径冲突 ⇒ 仅阅读。
· **MangoDisk**：其「**安全优先**的清理器」定位与本仓
  `scripts/check-disk.sh` 的门纪律同向（删除前必须证明是生成物）。
  ⚠️ 该仓的教训对本仓有直接价值：**清理器最大的风险是删掉不该删的**，
  与我们 2026-09-29 `cargo clean` 误删 115.2 GiB 的事故同源。

## Codewhale（MIT，41k★）

已于 2026-10-05 吸收其**授权栈**（第 2 层「同命令同时出现在 allow 与 deny ⇒ deny」、
第 7 层「repo-law 在 Full Access 下变硬 block」），落点在
`l6_meta/nt_approval.rs::action_verdict` 的注释与判据。
本轮**补入台账**（此前只在文档里被引用，未入 `repos.csv`）。

---

## 本轮修复的 NeoTrix 缺陷汇总

| 缺陷 | 位置 | 状态 |
|---|---|---|
| **审计抄录文件内容**（含密钥） | `l6_meta/nt_approval.rs::describe_action` | ✅ 已修 + 6 金丝雀锁 |
| **`action_verdict` 丢 AutoEdit 文件类白名单**（AutoEdit 下所有文件写被判 Ask） | `l6_meta/nt_approval.rs::action_verdict` | ✅ 已修（改为委托 `require_approval`，判据唯一真源）+ 3 锁（含穷举 mode×动作一致性不变量） |
| **`--sandbox` 完全无效**（写进无人读的单例） | `l3_embodiment/nt_sandbox.rs::init_sandbox` | ✅ 已修（同时推进 `global_shield()` 的活对象）+ `debug_assert` 两处一致 |
| `check_all` 与 `check_cli_command` 的 sandbox 闸**语义漂移** | `l3_embodiment/nt_shield_enforcer.rs` | ⚠️ **只记录不修**（需产品裁决，见该处注释三条理由） |
| `check_all` 的 sandbox 段**默认档不可达** | 同上 | ⚠️ 只记录（`SecurityGuard` 第 1 段对一切 `RequireApproval` 并提前 return） |

## ⭐ 三条可复用判据（本轮新增）

1. **审计/日志记摘要不记内容**（源：uber/ADR）。
   判据：能证明「是这份内容」即可，不必将内容抄进留存结构。
2. **遮蔽误伤的代价 = 审批失效**，比漏遮更难发现。
   ⇒ 遮蔽器必须配「不该遮的东西」的反向测试（`KEYBOARD=1` 就是这么抓到的）。
3. **解析/输出失败必须朝严格方向**（escalate / Err），不得回落成放行。
   （源：uber/ADR 的 `structured_risk_route_v1`，措辞可复用。）

---

# 追加：⛔ `asgeirtj/system_prompts_leaks` —— **判定不吸收**（2026-10-06）

用户给出 `Meta/muse-agent/workspace/system/system_prompt.md` 的直链。
**我核了仓库元数据，但刻意没有取那个文件的内容** —— 判定不需要读它。

## 实测事实（GitHub API，2026-10-06）
| 项 | 值 |
|---|---|
| License（仓库声明） | **CC0-1.0** |
| Stars | 68,974 |
| 内容性质 | 仓库自述：*"Documented system prompts from Anthropic - Claude Fable 5.1, Opus 5.5… OpenAI - ChatGPT GPT-6-Astra, Codex. Google - Gemini… xAI - Grok, Cursor, Kimi"* |
| 载荷来源 | **各厂商未公开的内部系统提示**，经泄漏/逆向整理 |

## ⛔ 判定：`NOT-CODE`

### 理由①：**声明的 CC0 管不到载荷**
CC0 是**整理者**对自己那部分（选材、编排、脚本）作出的弃权声明。
⛔ 他**无权**对 Anthropic / OpenAI / Google / Meta / xAI 的系统提示文本作出授权
—— 那些文本的著作权属于厂商，整理者只是**持有副本**。
⇒ 「仓库 LICENSE 写着 CC0」**不等于**「里面的提示文本可以自由取用」。
这与 `LICENSES.md` 既有口径一致：**先判载荷的权属，再判仓库的标签**。

### 理由②：来源是**泄漏**，不是授权发布
把泄漏出的厂商内部提示导入自家产品，属**规避技术保护措施 + 违反 ToS** 的高风险区。
⇒ 对 NeoTrix 而言，这是**产品级合规风险**，不是「多抄点素材」的取舍。
⚠️ 本仓正典已有 `Dark Forest` 公理与「无 LICENSE 只取设计」的纪律；
此源连「设计」都算不上 —— 它是**他人私有产物的副本**。

### 理由③：⭐ 按我**同一轮刚吸收**的 uber/ADR 纪律，它属于**不可信输入**
uber/ADR（Apache-2.0，Uber 生产）的硬规定：
*free-form model output is not copied into that trusted prompt slot*，
且 provenance（来源）是一等公民。
⇒ 一份**泄漏的、按构造就是「对手的私有提示」**的文本，
放进我们的 prompt 组装链，正是他们工程化防的那类东西。
⚠️ 有讽刺意味：**本轮刚落地「审计不抄内容」与「失败朝严格方向」，
转头就要引入一个来源不可信、权属不清的文本库** —— 自相矛盾。

### 理由④：不可证伪
即使抛开前三条，我们也无法验证这些文本**对应哪个版本、是否被改动过、
是否本身就掺了投毒内容**。而 prompt 恰恰是**最不该引入不可验证文本**的位置
—— 它直接决定模型行为，且绕过所有类型系统与测试。

## 我做了什么 / 没做什么
· ✅ 记入台账（526 条），标注 `#PAYLOAD-PROPRIETARY-LEAKED#NOT-CODE`
  ⇒ **目的是让下一个人不必重新评估**，而不是表示「以后可以考虑」。
· ⛔ **没有取那个文件的内容**，没有摘抄任何片段。
· ⛔ 没有基于它改动任何代码或文档。

## 若确实想要「系统提示架构」这类知识，正当路径
1. **用公开材料自己推**：厂商**公开文档**、已发表的 prompt 工程论文/博客。
   「系统提示应分层（身份/工具策略/拒绝边界/输出契约）」这类结论
   可以**独立推导并自行实现**，不需要看别人的私有文本。
2. **用明确授权的 prompt 工程仓**（MIT/Apache 且作者是内容作者）。
3. **审计我们自己的 prompt 组装点** —— 这才是同一意图下**我能立刻做的**：
   逐个 prompt 组装处标注 **provenance**（哪些是可信静态段、
   哪些来自模型/工具/用户 ⇒ 不可信），并确保**不可信段不进可信槽位**。
   这条与 uber/ADR 同源，且**完全在我方代码内**。
