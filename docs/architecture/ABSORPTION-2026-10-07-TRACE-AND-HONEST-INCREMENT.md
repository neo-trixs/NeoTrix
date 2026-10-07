# 吸收记录 —— 2026-10-07「轨迹页 + 诚实增量 + 门复活」批次

> 触发：用户给出 6 个外部源（`egoist/mygo` 及 5 个 hermes 系仓库 + `taste-skill`），
> 要求「用 NeoTrix 自己的能力，按最优解的进化路线，补齐缺陷，实现 neobot 的界面优化」。
>
> **本文件是这批吸收的裁决真源**：谁许可、取了什么、**没取什么**、以及每条裁决的证据。
> 规则依据：`NEOTRIX-STD-1.0.md` NTS-B10（记录真伪是唯一硬停）+ `AGENTS.md` §6。

---

## §0 先纠正一个前提：`egoist/mygo` 里没有 harness 界面

用户描述 `egoist/mygo` 为「用 AI 对着 DeepSeek harness 抄了一下界面，顺带复刻了流式对话页和轨迹页」。
**实测不成立**，逐项核实（2026-10-07，curl 打 API + raw，非记忆）：

| 检查 | 结果 |
|---|---|
| `GET /repos/egoist/mygo` | `description` = **"Develop desktop apps with a web frontend or native UI in Go"**；`language` = Go |
| `main` 分支全树 | 729 blob，**无** `harness` / `deepseek` / `trajector` / `chat` 命名的界面文件 |
| `examples/gallery/main.go`（原生 UI 展示） | grep `chat\|traject\|harness\|deepseek\|stream` → **0 命中** |
| AI 分支 `codex/native-ui-complex-apps` / `codex/native-ui-previews` / `native-ui` | 同上，0 命中 |
| 最近 100 条 commit message | 无 harness/deepseek/trajectory 相关 |

⇒ **结论**：mygo 是 Go 桌面框架（WebView 或原生 UI 双形态），不含复刻的 harness 界面。
本轮**没有从 mygo 取任何东西**（不是判断它不好，是它没有可取的目标）。
⛔ 记这一条是因为：若不纠正，后人会拿「mygo 抄了 harness」当成既成事实去找代码。

**真正可参照的是 DSH 自身的两页结构**（对话 / 轨迹）—— 已由 DSH 文档与生态克隆交叉证实，
本轮按它补齐 NeoBot。

---

## §1 许可裁决（NTS-B10 第 4 项：记录真伪）

全部逐个核实 **API 的 `license.spdx_id`** 与 **LICENSE 文件正文**，两者一致才判 MIT。

| 仓库 | SPDX | LICENSE 正文核对 | 判定 | 版权 |
|---|---|---|---|---|
| `Cranot/super-hermes` | `MIT` | 21 行完整 MIT 模板，含 `Permission is hereby granted, free of charge` | ✅ 可取码 | Copyright (c) 2026 Cranot |
| `Yonkoo11/hermes-dojo` | `MIT` | 标准 MIT 全文 | ✅ 可取码 | Copyright (c) 2026 Yonkoo11 |
| `outsourc-e/hermes-workspace` | `MIT` | 标准 MIT 全文；`package.json` 亦声明 MIT | ✅ 可取码 | Copyright (c) 2026 outsourc-e |
| `42-evey/hermes-plugins` | `MIT` | 标准 MIT 全文 | ✅ 可取码 | Copyright (c) 2026 Evey |
| `titanwings/colleague-skill` | `MIT` | 标准 MIT 全文 | ✅ 可取码 | Copyright (c) 2026 titanwings |
| `Leonxlnx/taste-skill` | `MIT` | 标准 MIT 全文 | ✅ 可取码 | Copyright (c) 2026 Leonxlnx |
| `egoist/mygo` | 未取 | — | — | 见 §0 |

⚠️ **`titanwings/colleague-skill` 已改名**：GitHub 重定向到 `titanwings/distilly`，
`README.md:9` 写「Formerly: Colleague Skill / colleague-skill」，**默认分支是 `dot-skill` 而非 `main`**
（`raw.../main/README.md` 返 404）。⇒ 复核许可时别按旧名与旧分支取。

**本轮实际取的是「做法」不是「代码」**：所有落地点都是本仓自写（`nt_run_trace.rs`、
`TraceView`、`classifyFailure`…）。上述仓库无一被 vendored，**无需贴 NOTICE**；
但凡日后真取码，必须按 MIT 保留版权与许可全文（记在此处是为了那时候不用重查）。

---

## §2 取了什么（逐条带出处与落地位置）

### 2.1 DSH 的两页结构 → NeoBot 的「对话 / 轨迹」页签

- **出处**：`deepseek-harness` 的会话视图分「对话」与「轨迹」两页。
- **落点**：`neobot-ui/src/neobot-root.tsx` 头部 `role="tablist"` + `nb-tab-chat` / `nb-tab-trace`；
  新增 `TraceView` 组件（独立文件级组件，不内联 —— 它有独立的拉取生命周期）。
- **数据源**：新增两条命令 `neobot_run_list` / `neobot_run_trace`。
- **⛔ 为什么此前是缺口**：`neobot_send` 返回里带着 `AgentRunResult.trace`
  （`nt_core.rs:60`），而界面只取 `r.output` ⇒ **整段被丢掉**；
  同时 `tasks`/`steps`/`file_changes` 三张表**没有任何读口**把它们端到端取出来。
  ⇒ 这是本仓第 N 次「导出 ≠ 接入」（R-P79），也是 STATUS §4 教训 29
  「只在终端接的能力 = 半个能力」的变体。

### 2.2 hermes-workspace 的**两档 stall 预算** → 本仓一档有界看门狗

- **出处**：`src/screens/chat/hooks/use-streaming-message.ts` 的
  `StreamLifecyclePhase` 与两档超时（accepted 120s / handoff 300s），
  理由是「服务端已受理但没动静」与「服务端在跑但忽然安静」**需要用户做不同的事**。
- **落点**：`RUN_STALL_SECS = 135`，配合 `run.started` / `run.stalled` 两条文案。
- **⛔ 为什么只有一档**：hermes-workspace 有两次往返（受理 + handoff），
  本仓的 `neobot_send` 是**一次同步 HTTP**（`nt_core::post_with_auth(.., 120)`）。
  阈值必须**略大于** 120s ⇒ 看门狗不会先于后端超时判死；
  否则用一个「猜的超时」盖住「已知的超时」，把可解释的失败换成不可解释的猜测。

### 2.3 hermes-workspace 的**人化工具标签** → 轨迹行的标签口径

- **出处**：`screens/chat/components/streaming-activity-ui.ts` 把工具名翻成
  `read foo.ts`（**只取 basename**，完整路径不进标签）、`exec <cmd 截到 27 字符 + …>`；
  `prompt-kit/tool-indicator.tsx` 折成「N tools used」。
- **落点**：`stepLabel()`（只留末段 + 特殊名映射）、`changeLabel()`（kind + 末段路径）。
- **理由（它自己的）**：转录该说**发生了什么**，不是**参数原文**；完整路径属于展开后的一层。

### 2.4 hermes-workspace 的**成本卡默认隐藏** → 本仓轨迹页**不显示任何用量**

- **出处**：`screens/dashboard/components/cost-ledger-card.tsx` 的注释自陈：
  把付费与「套餐内」混算曾产出「一个无意义的单一数字」，修法是**拆分**，且整张卡**默认隐藏**。
- **落点**：`nt_run_trace` 不变量 2（`ledger` 不以 `task_id` 为键 ⇒ 按时间窗摊到某轮是**猜**）
  + 界面侧 `{view === 'chat' && usageToday …}`（用量只在对应对话页）
  + 轨迹页常驻一句「用量按天统计，不摊到单轮」。
- **⛔ 这一条是独立到达同一结论的**：本仓从「账没有 task 键」推出「不给每轮费用」，
  hermes-workspace 从「混算产出无意义数字」推出「隐藏整张卡」⇒ 两边互为佐证。
  ⇒ 轨迹门把「不得出现用量数字 + 必须解释为什么没有」写成硬断言。

### 2.5 hermes-dojo 的**错误分类 + 明确「不是技能能修的」**

- **出处**：`hermes-dojo` 仓内 `analyzer.py` 的 `_classify_error_root_cause`，把错误分成
  `infra` / `auth` / `rate_limit` / `context_unavailable` / `security_policy` /
  `missing_parameter` / `unknown`，每类带 `fixable_by_skill: bool` 与一句 `suggestion`。
- **落点**：`classifyFailure()` + `failHint()`（7 类：unpaired / auth / rate / network /
  timeout / empty / other），失败气泡先给一句**可据以行动**的话。
- **⛔ 关键纪律**：判据全部落在**成对出现**的标记上
  （如 401/403 **且** auth|key|token|bearer 同时出现才算 auth）。
  理由取自 hermes-workspace 的 `connection-errors.ts`：它明确拒绝把泛化的 `token`
  字样路由到「重新登录」，因为 `"failed to fetch token from /api/x"` 是网络噪声。
  ⇒ **单个泛化词不构成证据**。

### 2.6 hermes-plugins 的**空白回复计为失败**

- **出处**：`evey-delegate-model/__init__.py::_call_model` 把「只思考不回答」的空内容
  计为失败并触发重试。
- **落点**：首发与重发两条路径都改：`.trim()` 后为空 ⇒ 走**失败**气泡（带「重发」），
  而不是 `t('chat.noOutput')` 那条**看起来正常**的「（无输出）」。
- **⛔ 为什么**：「没答上来」被说成「答上来了但是空的」，用户既不会重发也看不到红。

### 2.7 super-hermes 的**按行数省略**（而非按字符截断）

- **出处**：转录里的 `● [SYSTEM: …] (+27 lines)` / `(+60 lines)`。
- **落点**：`STEP_LINES = 6` + `trace.moreLines`（`还有 N 行`）+ 可逆的「收起」。
- **理由**：「多大」是信息；「前 200 个字符」是噪音，且丢掉了尾部。

### 2.8 taste-skill 的 `prefers-reduced-motion`（§6.B）

- **出处**：`taste-skill` 仓内 `SKILL.md` §6.B
  「Any motion above MOTION_INTENSITY > 3 MUST honor prefers-reduced-motion.
  Infinite loops … MUST collapse to static / instant under reduced motion.」
- **落点**：新文件 `neobot-ui/src/nb-reduced-motion.css`（独立文件而非散落各组件 ——
  判据是**结构**不是「记得写」）。
- **⭐ 一处刻意的非显然做法**：`animate-pulse` 在 reduce 下**不**写 `animation: none`，
  而是 `animation: none` **且** `opacity: 1`。因为 Tailwind pulse 的**起始帧是透明**，
  只关动画会让那个点**整个消失** ⇒ 用户在 reduce 下反而失去「正在跑」这个唯一线索。
- **顺带修的两条界面串**：`chat.noMemory` / `chat.historyPaused` 的英文文案里有两个
  em-dash（taste-skill §9.G 明令禁止），已改掉。

### 2.9 distilly 的**双通道输出契约** → 记为「已更强，不追」

- **出处**：`evey-telegram-ux` 每个 formatter 同时返回 `{html, plain}` + 一句
  `note` 告诉 agent 用哪个，「降级一次，绝不重渲染」。
- **裁决**：**NeoBot 已经有了**（`renderBot()` 渲染失败即回退纯文本 `<span>`）。
  ⇒ 记为「不追」并写明理由，避免下个窗口重新「发现」它。

---

## §3 明确**不取**什么（不是遗漏，是结论）

| 源 | 不取的东西 | 理由 |
|---|---|---|
| **taste-skill** | §4.7 hero / bento / logo 墙 / §5 滚动入场动画 / §10 模式词表 / §12 组件库 | **落地页**规则。NeoBot 是桌面工具界面：没有 hero、没有 bento、没有滚动入场 ⇒ 抄进来是**零消费方的死 CSS**。⚠️ 这条与 STATUS §1 记的「本仓已更强因而**不**照抄的 5 项」同源 —— 抄一份**更弱**的版本才是真风险。 |
| **taste-skill** | §9.D「Jane Doe 效应」的文案建议 | 面向营销页的假数据规避。本仓的真对应物是**「缺席 ≠ 空」/「不设静态默认」**，且已有 24 道门在守 ⇒ 更强。 |
| **super-hermes** | 5 个 prism 技能与 7 份参考 prompt | 那是**技能内容**（代码评审用的分析镜头），落点是 `skills/`，不是界面。与本轮目标无关。 |
| **hermes-workspace** | swarm / 多智能体编排（`swarm-*.ts` 15 个模块 + `agents/` 10 个角色） | 本仓范围是**单智能体桌面**。把它拉进来是扩范围，不是补缺陷。 |
| **hermes-workspace** | `local-session-store.ts`（JSON + 500 条上限 + 静默吞错） | ⛔ 与本仓的**「坏历史 = Err，拒猜」**（`nt_memory` 对照表的裁决）**直接冲突** —— 那一行是 `catch` 掉读写失败当没事。取它等于把一条已裁决的纪律推翻。 |
| **hermes-workspace** | `prompt-kit/tool.tsx` 的 4 态机 | **结构值得学、代码不必取**：本仓 `StepView` 已有 `ok: bool` + `tool` + `output`，
且 `nt_store` 的 `steps.ok` 就是这个语义。重铸一套类型只会多一个真源。 |
| **distilly** | 6 层人格栈、`tools/*.py` | 是**技能内容**（把人的素材蒸馏成 profile），与界面/架构无关。 |
| **hermes-plugins** | 分级 fallback（4 模型 × 3 次退避）、`SENSITIVE_PATTERNS` 本地路由 | 前者需要多 provider 目录（本仓是单配对核心）；后者本仓**本地优先**已天然满足。 |
| **mygo** | 全部 | 见 §0：没有目标。 |

---

## §4 顺带挖出的四个**既存**缺陷（不是本轮引入）

### 4.1 四道门读的是**已搬走**的命令注册表（本仓第 6 次「搬家后下游没跟」）

2026-10-02 的 P0 复盘把注册表从 `src/main.rs` 搬进 `src/lib.rs` 的 `neobot_commands!` 宏
（`lib.rs` 段头写明理由：`tests/` 够不到 bin 侧 ⇒ 漏注册无信号）。
**四道门仍读 `main.rs`** ⇒ 抽出的注册集合近乎为空：

| 门 | 改前的观测 | 改后 |
|---|---|---|
| `nt_check_api` | 「Rust 注册 **0**」+ 70+ 条**谎报** + `ReferenceError: f is not defined` **崩掉** | 注册 **76** · PASS |
| `nt_check_ui_calls` | 注册近乎为空 ⇒ `write_clipboard_text` 等被报「未注册」 | 注册 76 · PASS |
| `nt_check_ship_ui` | 同上 | PASS |
| `nt_check_status` | 把 §1.3 的 **18 条真实命令**全报成「后端不存在」 | PASS |

⛔ **危害方向**：门长期红 ⇒ 人会去**改文档去迎合错的门**
（把真实的 18 条命令从 STATUS 里删掉）⇒ 那就是让文档开始说谎。
本轮实测：`nt_check_api` 在**真有缺口**时不是报缺口，而是抛 `ReferenceError` 直接退出 ——
**门在最该说话的时刻说不出话**。

### 4.2 五道门不剥注释（同一缺陷，第 5 次）

`api` / `ui_calls` / `ship_ui` / `status`（+ 我自己写 `trace` 时踩了两次模板字符串反引号）
都把**注释里的字**当成代码/数据：

- `api.rs` 的 note 位置上有 8 行块注释 ⇒ `ApiSpec::new("remote_bridge_ping", …)` **整条解析不到**
  ⇒ 契约条目少 1 条、`upstream_total` 对账报「95 != 96」**假警**（真值 96）。
- ⛔ **剥注释不能朴素做**：`api.rs` 的 note 里有**字面量 `//`**（`file://`、`pet://status`），
  `replace(/\/\/.*$/gm,"")` 会把它们截成 `file:` / `pet:` ⇒ **门改掉了它本该校验的数据**。
  ⇒ 已写成字符串感知（含 `\"` 转义、`/* */` 嵌套计数）的版本。

### 4.3 两道门指向**冻结且从不构建**的树 ⇒ 从来没跑过

`nt_check_layout` / `nt_check_interact`（及 `nt_shot` / `nt_shot_state`）的 `DIST` 指向
`apps/neobot-desktop/frontend/dist`：

- `AGENTS.md` §0：那是**参考树、不可交付**；
- `.gitignore` 第 334 行**显式忽略** `apps/neobot-desktop/frontend/dist/` ⇒ CI 里**永远不会被构建**。

⇒ 这两道门**一次都没跑起来过**，而 `STATUS.md` §1.5 把它们列在「门禁」里、§5 还写「已闭合」。
⭐ 佐证「本意就是交付树」：**门里的桩注册的是 `neobot_convo_list` /
`neobot_core_capabilities` / `neobot_usage_summary` / `neobot_memory_list`** ——
这四个**只存在于 `neobot-ui/` 的自持根**，vendored 树没有这些调用。**只是路径忘了改。**

复活后它们立刻各找出 4 条真问题（桩停在 `_page` 改名之前、选择器是四层后代链、
CSS 类名还是上游的 `.dsh-pet__sprite`、`convo_id` 断言方向反了）。

### 4.4 `ipc_roundtrip` 在 HEAD 上就是红的（2/5）

`cargo test -p neobot-desktop` **在 HEAD 上即失败**：两条测试断言
`neobot_convo_messages_page` 返回**裸数组**，而它自 `70a592df` 起返回 `{messages,hasMore,nextSeq}`。
⛔ 已用 `git stash` + 重跑**证实非本轮引入**（同一两条、同样的错值）。
⇒ 「cargo test 全绿」这句话在本轮之前**不成立**；这也解释了为什么 4.3 里那些门坏了这么久没人管
—— **测试长期红 ⇒ 没人看它的输出 ⇒ 它真正该抓的东西一起被忽略**。

---

## §5 本轮新增的门：`nt_check_trace`

stub-boot 跑**生产包**，逐条断言九件事（理由见该文件头）：

1. 三态可辨（还在读 / **读不到** / **真的没有**是三句不同的话）
2. 轮次行渲染，且 `status` **原样透传**（含界面不认识的 `quantum`）
3. 失败步数 > 0 才用危险色（0 个失败时显示「0 失败」是噪声）
4. 展开真去调 `neobot_run_trace`，并画出每一步 + 改了哪些文件
5. 长输出按**行**折叠成「还有 N 行」，可展开、也可**收起**
6. ⭐ **轨迹页任何位置都没有用量数字**，且必须有一句解释为什么没有
7. 跑轮信号把「正在想…」换成「开始跑…」
8. 零 JS 异常

⭐ **变异验证（必须，否则门是空气）**：把「用量只在对应对话页」的条件改成两页都显示
⇒ 门**第一次没抓到**（全绿）。原因是断言只量了 `nb-trace` **面板**的文本，
而那个数字渲染在**头栏** ⇒ **量错了作用域**。
⇒ 已改成量**整棵自持根**，再跑同一变异即变红；还原后复跑绿，且 `diff` 与变异前**逐字相同**。

> 「门犯的错和它要抓的错是同一种」在这里又是同一个形状：
> 门要抓「这一页出现用量数字」，而它量的是「面板里出现用量数字」。

---

## §6 留给下一个窗口的欠账（**明确不做，不是忘了**）

| # | 项 | 为什么现在不做 |
|---|---|---|
| 1 | 5 道门各写一份剥注释实现 | 应抽成 `scripts/ops/lib/strip-comments.mjs` 共用，但那要动 5 个文件的 import，风险大于本轮收益 |
| 2 | 真 SSE token 流 | `neobot_send` 走配对核心的 `POST /v1/agents/run`，**那个端点是否支持 SSE 本仓无法验证**。库里**有**真流式（`nt_http_engine::run_turn_stream` + `nt_agent::run_local_turn_stream_as`），但**当前这条发送路径够不着它**。⇒ 按「不可用就不渲染」，先发能证实的生命周期信号。⚠️ `neobot-ui/src/ipc.ts` 早先否掉事件流协议时写的理由是「后端没有可流的东西」——**那句不完全准确**，别照抄 |
| 3 | vendored 参考树里那份**过时的** `neobot-root.tsx` 副本 | 它调的是**已删除**的 `neobot_convo_messages`。⭐ 但 `nt_api_contract.py`（2026-10-04）已裁决过：冻结树的调用**只报告不当失败** ⇒ 照此办理，`api`/`ui_calls` 两门已按此出 ℹ️。⛔ 不要去删它（那是冻结快照） |
| 4 | `steps` 没有时间戳/耗时 | 轨迹页因此**不能**显示「何时/多久」。补它要改 `steps` 建表 + `add_step` 签名 + 全部调用点 ⇒ 结构性改动，另立一轮 |
| 5 | 主题 token 双档**结构性**对齐 | `theme.css` 的深浅两档是手写两块，`theme.css` 自己的注释记着「只补浅色会让深色回落到字面值 ⇒ 门也抓不到」。hermes-workspace 的 `theme.ts` 用**双向穷尽 `Record`** 让「只发一个变体的主题」编译不过。⇒ 本轮未做，需连门一起改 |
| 6 | taste-skill §4.11「Page Theme Lock」与本仓「气泡固定内容色」的冲突 | 本仓**故意**让气泡在深浅两档都固定浅色（有注释说明可读性理由）。taste-skill 会说这是「深色页里夹一块浅色」。⇒ **这是设计分歧不是缺陷**，需要产品判断，不该由 agent 单方面改 |