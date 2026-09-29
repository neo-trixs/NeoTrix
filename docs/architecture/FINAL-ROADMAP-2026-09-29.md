# NeoTrix 最终进化路线清单 — 45 仓吸收定稿（2026-09-29）

> **本文件是四轮吸收的最终定稿**：09-27（109+385 仓）· 09-28（8 源）·
> 09-29 第一批（30 源）· **09-29 第二批（本文件新增 15 仓）**。
> 排期与状态以本文件为准。
>
> **核心结论未变，且本轮再次证实**：
> **「自进化机制不是护城河，能证明自进化是否有效才是」** ——
> 已从 11 仓扩到 **45 仓**验证。**例外只有 4 个**（见 §2），
> 且其中最有价值的那个（`qybaihe/mu`）**公开了自己失败的数字**。

---

## 1. 本轮 15 仓的取证结论

### 1.1 ⛔ 先证伪 5 个提交前提

| 提交的前提 | 实测真相 |
|---|---|
| `excel-codex-bridge` 桥接「Excel ↔ Codex」，有公式/单元格/图表 schema | **假。** 全仓 `formula\|worksheet\|cell\|chart\|openpyxl` 只有 **5 处命中，全在 `excel_signin.py`**，且只用来生成空白 `.xlsx` 触发加载项面板。**零公式、零单元格、零图表表示法。** 它桥的是「Codex CLI ↔ ChatGPT-for-Excel 加载项的私有后端」 |
| `BongoCat` 做「输入注入」 | **假。只捕获，不注入。** `SECURITY.md` 明文否认注入。唯一 `SendInput` 在一个 `EXCLUDE_FROM_ALL` 的测试工具里 |
| `typesafe-computer-use` 的「类型安全」= 工具 schema 类型系统 | **假。** `typesafe` 是**供应商名**，「type」指分类任务的有标签选项。**无类型检查器、无 schema 类型层。** ⛔ 吸收时**不要连名字一起吸收**，否则读者会误以为工具 schema 被类型检查过 |
| `Aegis` 是安全/防御/agent 防护 | **都不是。** 是 prompt/workflow 方法包 + 测评基础设施。**零 CVE、零输入净化、零沙箱、零权限模型** |
| `aether-search` 有自建索引/排序 | **无自建索引。** RRF 有，但是 `fusion.py` **全文 22 行逐字移植**自 gigaxity（标准 k=60，无任何改造）—— **这是引文不是机制** |

### 1.2 🔴 溯源红旗（⛔ 需澄清后才能进正典）

| 仓 | 问题 |
|---|---|
| `qybaihe/mu` | GitHub `fork: False`、`parent: null`、全部提交作者 `qybaihe`，但 `LICENSE` 首行 `Copyright (c) 2025 Mario Zechner`。README 自称建于 `earendil-works/pi`(110k★) 与 `AionUi`(33k★) 之上并已署名。内部 `10-rename-to-mu.md` 记载 2026-09-21 从 kyrn 改名 ⇒ **代码最多 6 天历史**。未能定位原始仓（`badlogic/mu` → 404）。README 用复数 "authors" 与单一提交作者不符 |
| `GanyuanRan/Aegis` | `LICENSE` 双署名（`Jesse Vincent` / `Ganyuan Ran`），但 README/CONTRIBUTING/NOTICE **零处**提及与 Jesse Vincent 的关系 |

**⇒ 两者都可吸收设计**（MIT 无来源限制），但**不应进正典索引**直到署名链澄清。

### 1.3 ⛔ 零吸收（1 仓）

**`aimeoa/hanshuang-codex`** —— 仓库整体标 MIT，但 `codex-skills/` 下有
`crack-keygen` / `full-crack` / `edr-bypass-re` / `dma-attack` / `anti-cheat` / `game-cheat`。
**MIT 声明覆盖不了分发破解/EDR 绕过教学内容的权利**，也不改变其违法或违反各 agent 服务 ToS 的事实。
**这台机器不能吸收这个仓的任何东西。**

**唯一可记的教训**（而且它是关于**我们**的）：
> `codex-skills/full-pentest/SKILL.md` 开头必须专门写一段「唯一禁止项」保护作者自己的 API 端点
> ⇒ **注入后行为失控，护栏必须建在���注入体之外**。
> 这与 `gpt-cox`（shell 注入到 agent 配置）说的不是一回事，但结论一致：
> **把控制权交给提示词/注入，等于放弃控制。**

**识别「内容仓」的方法**：`.md` 占比 > 50%（该仓 76%，3,615/4,749）且 commit 数极低（27）。

---

## 2. 🔴 本轮最重要的交叉发现：我们的 `nt_judge` 抄的是 mu 自测**最差**的那个决策点

`qybaihe/mu` 的 `kyrn/docs/08-jev-retrospective.md` 是四轮吸收里**唯一一份公开自己失败数据**的回测报告：

> 「Jev 已有局部正向作用，但当前记录不足以证明整体价值很大或净收益为正。」

**它对 admission（chunk 准入）决策点的实测**：

| 指标 | 数值 |
|---|---|
| 批数 / 块数 | 194 批 / **2,412 块** |
| 分类结果 | 2,385 result · 6 error · 21 unknown |
| **`drop=true`** | **0 —— 一个块都没删掉** |
| 消耗 | **1,966,585 input tokens = 已记录 Jev 输入的 54.0%** |
| 延迟 | 每批 p50 **1,351ms**、p95 **3,807ms** |
| 原文结论 | *"这不是「Jev 应该删掉源码」的证据……**不是为了制造节省而放宽删除阈值**。"* |

### 而我们的 `nt_judge.rs` 正是 admission

**实测确认**（非文档转述）：
- `neotrix-core/src/l0_substrate/nt_judge.rs:1` 「EVO-02 mu 式 judge-kernel」
- `nt_judge.rs:6` **「无 I/O、无时钟调用、无线程」**
- `nt_judge.rs:16-17` `DEFAULT_SENSITIVE_TERMS` + 长度阈值
- `sessions/handoff-evo-20260926.md:68` 把 **「admission/墓碑/ledger」列为 P0**

⇒ **我们抄的是一个「消耗 54% 判断输入、收益为零」的决策点。**

**但要公平地说清两件事**：
1. **我们的实现是纯规则**（敏感词 + 长度），**没有真调小模型** ⇒ 上面那些 token/延迟**不适用于我们**。
2. ⇒ 真正该吸收的是 **mu 自己的方法论**（见 §4 M-1/M-2/M-3），**以及「先测量再扩建」这条纪律**。

⛔ **动作**：把 `handoff-evo-20260926.md:68` 的 P0 表述从「做 admission」改为
**「先接线一个 mu 证明有效的决策点（技能隐藏/蜂群过滤），admission 降级为观察项」**。

---

## 3. 本轮 45 仓的「是否测量自己」总账

| 状态 | 仓 | 备注 |
|---|---|---|
| ⭐ **真 A/B 且在 CI 门内** | `Aegis`（+27.27pp，n=44，CI 宽 36pp）· `qwen-audio-agent`（5 维不变量）· `qybaihe/mu`（**自曝失败**）· `dsh-use-wallpaper`（**919 项全绿 + 真数据**） | 4 个 |
| ◐ **有评测设施但门槛留空** | `rovai-ai`（`minimumQuality: null` + 自述"权重未标定"）· `Varen-AI-CAD`（11 任务只跑 3，`parts_count` 全 0） | 2 个 |
| ❌ **零测量** | `PI-Desktop`（6k★）· `Agentero`（822★）· `DEEIX-Chat`（1.5k★，353k 行）· `hanshuang-codex` · `aether-search` · `EasyAntigravity` · `BongoCat` · `excel-codex-bridge` · `typesafe-computer-use`（唯一例外，见下） | 9 个 |
| ✅ **有测量但无主张** | `BongoCat`（2,730 断言，没提可量化主张）· `kev`（前轮） | — |
| ⚠️ **有但窄** | `typesafe-computer-use`（`benchmarks/osworld/*.jsonl` **append-only**，含 `git_commit`+`git_dirty`；**但 n=10 行、4-6 个唯一任务、无基线对照**） | — |

> **⇒ 41 个仓里只有 4 个真在测量自己对外宣称的东西**，其中一个是 4 star 的最小项目。
> `DSW-NOTE`：`qwen-audio-agent`（2.8k★）测的**是不变量**，不是性能 ——
> README 宣称的「full-duplex / 自然打断」**没有任何延迟数字**。

---

## 4. 本轮新增的可吸收机制（14 条，按可抄性排序）

### 4.1 🟢 立刻可抄（低风险高回报，共 6 条）

| ID | 机制 | 来源 | 落点 | 验收 |
|---|---|---|---|---|
| **M-1** | **臂中立 veto 契约**：评分器**完全不知道**跑的是哪个臂；**7 种否决而非打分**（`workspace-change` / `verification-failure` / `false-completion-claim` / `destructive-tool-use`…）；扫输出是否泄漏臂标签防作弊 | `Aegis` `tests/helpers/score_agentic_benchmark_outcome.py:28-62` | 新建 `crates/nt-core-capability-tree/tests/arm_neutral_contract.rs` | 同输入两臂跑，评分器源码零分支差异 |
| **M-2** | **`cacheImpact` 三分类**：`none` / `append-only` / `prefix-mutating`。原文：*"prefix-mutating decisions … should only be applied at cache boundaries"* | `qybaihe/mu` `packages/kyrn-judge/src/decision.ts:36-41` | 任何决策点定义处 | 分类为 `prefix-mutating` 的不得在非缓存边界生效 |
| **M-3** | **choice-escape 类型不变量**：`defineDecision()` 在**定义时**抛 `TypeError` —— 任何 `choice` 问题**若无 escape 选项**（`none`/`other`/`unclear`/`unknown`）⇒ 小模型永远无法被迫从封闭集瞎选 | `mu` `decision.ts:69-88` + `policy.ts` `hasEscapeOption()` | 同 M-2 的注册处 | 定义一个无 escape 的 choice ⇒ 编译期/注册期红 |
| **M-4** | **影子→active 升级路径**：新决策点**默认 shadow**（调 judge、记 ledger、但**返回 fallback**），用 `judged` vs 真实 `outcome` 对比后再切 active | `mu` `decision.ts:22-27, 91-110` | `nt_judge.rs` 已是影子模式 ⇒ **只差「记录 judged」这一步** | ledger 每条同时含 `judged`/`outcome`/`source` |
| **M-5** | **审批 = 数据库行 + 原子事务 + 进程纪元栅栏**：`BEGIN; plan_approvals: pending→approved; sessions.mode: plan→agent; append audit; COMMIT`。超时=拒绝（fail closed）；**进程重启时 pending 标 `interrupted`、拒绝所有旧响应** | `PI-Desktop` ADR 0052 §4 | `crates/neotrix-neobot/src/nt_store/` | 进程重启后旧审批响应**必须**被拒 |
| **M-6** | **上下文槽位 IR + 纯函数装配**：`Assemble()` 返回 `(messages, AssemblyTrace)`；`Trace.TrimmedSlots` 记录**裁剪决策是返回值的一部分，不是副作用** | `DEEIX-Chat` `context_assembler.go:104-158`（Apache-2.0） | `nt_core_context/context_budget.rs`（**当前完全死**） | 装配超预算 ⇒ trace 里有名有姓，不是静默丢弃 |

### 4.2 🟡 需改造后吸收（4 条）

| ID | 机制 | 来源 | 落点 | 验收 |
|---|---|---|---|---|
| **M-7** | **两级熔断 + 半开探针租约**：模型级 `failures≥5/15min/3min`、上游级 `failures≥20 且 模型数≥3/30min/5min`；**半开窗口 30s 只授予一个探针租约**（其余 `half_open_denied`）防惊群；探针失败**立即**重开不等阈值 | `DEEIX-Chat` `channel.go:78-91`（Apache-2.0） | `neobot` 的 provider 调用层 | 10 并发打熔断中的 provider ⇒ **恰好 1 个**拿到探针 |
| **M-8** | **turn generation 栅栏**：单调 `turnSequence`；每个事件带 `{turnId, turnGeneration}`；`ctx.turnGeneration < committed` 即丢弃。**把「代」做成数据而不是隐式状态比较** | `qwen-audio-agent` `realtime-turn-state.mjs`（156 行） | 流式输出 / 异步任务回灌 | 迟到的 `audio.delta` **必须**被丢弃 |
| **M-9** | **「本地超时 ≠ 远端已释放」**：四相 `idle→pending→cancelling→idle`；`waitMs` 到期→`cancel()`；`cancelMs` 到期→**直接 `disconnect()` 强制释放** | 同上 `realtime-response-slot.mjs`（46 行） | 同上 | 远端卡住时本地不会死等 |
| **M-10** | **独占分配守卫**：两个 Feature 匹配到**同一** BRep 候选时，**不求最优匹配**（不做二分图），而是 `len(group)>1` ⇒ **全部降级 AMBIGUOUS + 清空 selected + `usable_for_verification=False`** | `Varen-AI-CAD` `resolver.py:684-709`（**AGPL ⇒ 只取设计重写**） | 任何「多来源声称同一目标」的校验 | 两个 claim 撞同一目标 ⇒ 双方都不可用 |
| — | **「payload 是数据，永远不是能力」** | `PI-Desktop` ADR 0040 | 直接写进架构公理 | 总线不能洗白权限 |

### 4.3 🔵 值得吸收为「文档/纪律格式」（3 条）

| ID | 机制 | 来源 |
|---|---|---|
| **M-11** | **判据失去鉴别力 → 显式记录并换代理判据**：作者发现全屏 p99 两侧都钉在 255 ⇒ *"**这不是 bloom 失效**，而是该判据口径不再有鉴别力"*，改用「每帧 render 提交 ≤33.3ms」 | `dsh-use-wallpaper` `technical-notes.md §4` |
| **M-12** | **限制四段式**：每条限制写 `Retained Item` / `Retention Reason` / `Observation Metric` / `Retirement Trigger` | `Aegis` `docs/current/AEGIS_KNOWN_LIMITATIONS.md` |
| **M-13** | **主路径配置+断言语义分层**：shadow 判别与 `outcomes` 断言 | `Aegis` `scripts/aegis-shadow.ts` |
| **M-14** | **已知失败棘轮 + 归零纪律**：`known-failures.json` **已归零但机制保留** | `dsh-use-wallpaper` `check-known-failures.mjs` |

### 4.4 ⛔ 本轮明确不吸收

| 模式 | 来源 | 理由 |
|---|---|---|
| **正则 denylist 当安全门禁** | `EasyAntigravity` | `[\s\S]*` 贪婪跨行是**结构性**误报源（两个 token 同现必命中，哪怕顺序相反）；且 `bash -c "$(echo X\|base64 -d)"` 可绕过。⛔ 但它的**处方式阻断文案**（`root_cause` + `destructive_impact` + `safe_alternative` 三字段各 `min(5)`）值得抄 |
| **预编译未签名二进制** | `EasyAntigravity` `assets/EasyAG-Resident.exe` | 一个主张自己更可信的安全工具，供应链上却有未签名 exe |
| **从别的 App 的 WebView2 缓存挖凭据** | `excel-codex-bridge` | 即便设计得当，这是我们**不该照抄**的模式 |
| **prompt 承载协议** | `excel-codex-bridge` | 账单是 53 行的 `_repair_invalid_json_backslashes()` 去猜模型意图 |
| **AGPL 代码** | `Varen-AI-CAD`（AGPL-3.0）· `BongoCat`（AGPL-3.0） | 只取设计 |
| **LGPL 代码** | `PI-Desktop`（LGPL-3.0） | 只取设计（309 篇 ADR 的质量值得读，代码不抄） |
| **「声明即自动授予」当权限模型** | `PI-Desktop` ADR 0008 自陈 | `Declared manifest permissions are still auto-granted at load time`，且只覆盖 `pi.*` 不覆盖 `require("node:fs")`。⛔ **不要当沙箱引用** |
| **单文件 54k 行** | `rovai-ai` `db.rs` | 373k 行压在 171 文件里 |
| **「限制清单」写成散文** | `Agentero` `AGENTS.md` 开发规则段混入平台踩坑 | |

---

## 5. 最终路线清单（与前四轮合并，按批次）

> 批次 A/B/C/D/E 的完整条目见 `FEATURE-MAP-TASKS-2026-09-29.md`（特性级清单）。
> 本节只给**本轮新增**的、以及**因本轮发现而必须改判**的。

### 批次 A（零 `.rs`，~2 天）—— 下一步仍是 A3

| 状态 | 任务 |
|---|---|
| ✅ | A1 CI 幻影门 · A2 未编译代码可见 |
| ⬜ **A3** | `check-test-baseline.sh` 账本填充（**当前 0 字节**） |
| ⬜ **A4** | `nt_manifest.py` 查数字（kev `verify_claims.py`） |
| ⬜ **A5** | `skill_loader` policy 接线（三个 `visible_*` 零消费者） |
| ⬜ **A6⬛新增** | **改判 `handoff-evo-20260926.md:68`**：admission 从 P0 降为观察项（依据 §2） |
| ⬜ **A7⬛新增** | **`Aegis` 臂中立 veto 契约**（M-1）—— 本仓第一个真正臂中立的东西 |
| ⬜ **A8⬛新增** | **限制四段式**（M-12）套用到本仓 15 条 ⛔ 需裁决项 |

### 批次 B（测量面）

| 状态 | 任务 |
|---|---|
| ⬜ | B1 门可满足性元门 · B2 env 指纹 · B3 证伪门 · B4 maturity 降级 |
| ⬜ **B5⬛新增** | **`mu` 的影子→active 升级**（M-4）—— `nt_judge` 已是影子，**只差记录 `judged`** |
| ⬜ **B6⬛新增** | **决策点接真实小模型**（`mu` 的 Jev/Laya）—— 但**先只接 mu 证明有效的两个**（技能隐藏/蜂群过滤） |

### 批次 C（策略面，⚠️ 并发冲突）

| 状态 | 任务 |
|---|---|
| ⬜ | C1 三态工具策略 · C2 StopReason · C3 策略地板 · C4 DNS · C5 MCP |
| ⬜ **C6⬛新增** | **两级熔断 + 半开探针租约**（M-7，Apache-2.0 可直接实现） |
| ⬜ **C7⬛新增** | **审批 = DB 行 + 原子事务 + 纪元栅栏**（M-5）—— 改判 `nt_policy` 的内存标志位形态 |

### 批次 D（记忆与检索）

| 状态 | 任务 |
|---|---|
| ⬜ | D1–D5（前四轮） |
| ⬜ **D6⬛新增** | **上下文槽位 IR + trace**（M-6）—— 落 `context_budget.rs`（**当前完全死**，正好是复活点） |
| ⬜ **D7⬛新增** | **`cacheImpact` 三分类**（M-2）+ **choice-escape 不变量**（M-3）—— 与 D 组配合，因为两者都改 context 装配 |

### 批次 E（桌面回路）

| 状态 | 任务 |
|---|---|
| ⬜ | E1–E4（前四轮） |
| ⬜ **E5⬛新增** | **turn generation 栅栏 + 超时阶梯**（M-8/M-9）—— 流式/异步回灌必踩 |
| ⬜ **E6⬛新增** | **可逆性感知置信度门控**（`typesafe` `Decision.confidence`）—— *"只有命名目标的答案会降低置信度：点击落在某处，错的某处不会被撤销"*. **不确定只在动作不可逆时终止循环** |

---

## 6. ⛔ 需裁决（本轮新增 3 项，累计 9 项）

| # | 事项 | 为什么不能自己决定 |
|---|---|---|
| 7 | **`qybaihe/mu` 署名链**（Mario Zechner / qybaihe / 上游关系） | 未澄清前不进正典索引 |
| 8 | **`Aegis` 与 Jesse Vincent 的关系**（双署名但零提及） | 同上 |
| 9 | **`Aegis` 的 22 个 skill 要不要吸收** | 与 superpowers **大面积同名同义**（brainstorming / TDD / systematic-debugging / using-git-worktrees…）⇒ 吸收会造成本仓 skill 目录冲突。⛔ 只取测评方法论 + 规则分层标尺 |

---

## 7. 本轮学到的两条方法论

1. **⭐ 最有价值的吸收来自「公开自己失败」的仓，不是「star 最多」的仓。**
   `mu`（313★）的回测报告比 `DEEIX-Chat`（1.5k★，353k 行，零测量）有用一个数量级。
   ⇒ **选题时先问「它有没有一份说自己哪里不行的文档」**，再问 star。

2. **⭐「本地全绿但不可交付」的第二个教科书案例：`Varen-AI-CAD`。**
   它的 resolver/gate 设计非常好（签名-证据-对应物三层、独占分配守卫），
   但 **11 个声明任务只跑了 3 个、`parts_count` 全 0、`.step`/`.stl` 文件数 0**、
   所有 artifact 路径是作者本机 `G:\...` ⇒ **外部无法复现**。
   README 主打的减速器/变速器**只存在于图片链接和 CHANGELOG 散文里**。
   ⇒ 这与 `LESSONS-20260928-verification-must-be-executable` 是同一个病。

---

## 8. 复核命令

### 8.1 许可证与溯源（**2026-09-29 全部经 GitHub API 实测**）

| 仓 | License | fork | parent | created | stars | 判定 |
|---|---|---|---|---|---|---|
| `qybaihe/mu` | MIT | **False** | **None** | 2026-09-22 | 313 | ⚠️ 署名链待澄清 |
| `GanyuanRan/Aegis` | MIT | False | None | 2026-04-30 | 1,301 | ⚠️ 双署名待澄清 |
| `vanyu0710/Varen-AI-CAD` | **AGPL-3.0** | False | None | 2026-08-03 | 34 | ⛔ 只取设计 |
| `DEEIX-AI/DEEIX-Chat` | **Apache-2.0** | False | None | 2026-05-21 | 1,510 | ✅ 可抄码 |
| `poco-ai/Agentero` | MIT | False | None | 2026-07-07 | 822 | ✅ |
| `murray17/rovai-ai` | MIT | False | None | 2026-07-17 | 114 | ✅ |
| `QwenAudio/qwen-audio-agent` | **Apache-2.0** | False | None | 2026-07-27 | 2,809 | ✅ |
| `vastsa/pi-desktop` | **LGPL-3.0** | False | None | 2023-03-22（重写项目） | 6,048 | ⛔ 只取设计 |
| `DSDS-CMHL/EasyAntigravity` | MIT | False | None | 2026-09-16 | 123 | ✅（但机制不抄） |
| `vladelaina/BongoCat` | **AGPL-3.0** | False | None | 2026-08-22 | 3,154 | ⛔ 只取设计 |
| `awlevin/typesafe-computer-use` | MIT | False | None | 2026-09-16 | 1,074 | ✅ |
| `Kaixxrua/excel-codex-bridge` | **Unlicense** | False | None | 2026-09-24 | 157 | ✅ 但**模式不抄** |
| `yu502950715yang/dsh-use-wallpaper` | MIT | False | None | 2026-08-17 | **4** | ✅ ⭐测量最诚实 |
| `aimeoa/hanshuang-codex` | MIT | False | None | 2026-09-06 | 717 | ⛔ **零吸收** |
| `maliaosaide/aether-search` | **`NOASSERTION`** | False | None | 2026-07-27 | 13 | ⚠️ 有 LICENSE 文件但 GitHub 未识别（研究代理读到的是 MIT）⇒ 抄设计前需自查 |

**`fork=False` + `parent=None` 全部成立** ⇒ 无一是从 GitHub fork 而来。
**⇒ `qybaihe/mu` 的署名问题不是「fork 清洗」，而是「非 GitHub 血统的搬运」** ——
这加强了 §1.2 的红旗：它 5 天大、单作者、来源链不清，却挂着 `Mario Zechner` 的版权。

### 8.2 交叉发现（§2）的证据

```bash
sed -n '1,20p' neotrix-core/src/l0_substrate/nt_judge.rs    # 「无 I/O、无时钟调用、无线程」
grep -rn "mu 式" --include="*.rs" neotrix-core/src | head   # 2 处
grep -n "admission" sessions/handoff-evo-20260926.md         # :68 把 admission 列为 P0
```

### 8.3 已有门

```bash
bash scripts/check-ci-refs.sh --strict; echo $?
bash scripts/check-truth-surface.sh 2>&1 | sed -n '3p'
```

### 8.4 溯源自查（若装了 gh）

```bash
gh api repos/qybaihe/mu --jq '{fork,parent:.parent.full_name,created_at,license:.license.spdx_id}'
gh api repos/maliaosaide/aether-search --jq '.license'
```
